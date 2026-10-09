use crate::ssh::*;
use std::path::{Path, PathBuf};

fn host() -> String {
    std::env::var("SSH_TEST_HOST").unwrap_or_else(|_| "127.0.0.1".into())
}

fn port() -> u16 {
    std::env::var("SSH_TEST_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(2222)
}

fn user() -> String {
    std::env::var("SSH_TEST_USER").unwrap_or_else(|_| "tester".into())
}

fn key_path() -> PathBuf {
    if let Ok(p) = std::env::var("SSH_TEST_KEY") {
        return PathBuf::from(p);
    }
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    [
        manifest.join(".sandbox/id_ed25519"),
        manifest.join("../.sandbox/id_ed25519"),
    ]
    .into_iter()
    .find(|p| p.exists())
    .expect("could not find .sandbox/id_ed25519; set SSH_TEST_KEY")
}

fn connect() -> SshConnection<(String, u16)> {
    SshConnection::from_private_key(user(), key_path(), (host(), port()))
}

#[tokio::test]
async fn runs_a_command_and_captures_stdout() {
    let mut conn = connect();
    let out = conn.execute_command("echo hello").await.unwrap();
    assert_eq!(out.stdout, "hello\n");
    assert_eq!(out.stderr, "");
}

#[tokio::test]
async fn captures_stderr_on_success() {
    let mut conn = connect();
    let out = conn
        .execute_command("echo oops >&2; echo fine")
        .await
        .unwrap();
    assert_eq!(out.stdout.trim(), "fine");
    assert_eq!(out.stderr.trim(), "oops");
}

#[tokio::test]
async fn nonzero_exit_returns_error_with_code_and_stderr() {
    let mut conn = connect();
    let err = conn
        .execute_command("echo bad >&2; exit 3")
        .await
        .unwrap_err()
        .to_string();
    assert!(err.contains("exited with 3"), "unexpected error: {err}");
    assert!(err.contains("bad"), "stderr missing from error: {err}");
}

#[tokio::test]
async fn unknown_command_is_an_error() {
    let mut conn = connect();
    let err = conn
        .execute_command("definitely-not-a-real-command-xyz")
        .await
        .unwrap_err()
        .to_string();
    assert!(err.contains("exited with 127"), "unexpected error: {err}");
}

#[tokio::test]
async fn session_is_created_lazily_and_reused() {
    let mut conn = connect();

    conn.execute_command("true").await.unwrap();

    for i in 0..5 {
        let out = conn.execute_command(&format!("echo {i}")).await.unwrap();
        assert_eq!(out.stdout.trim(), i.to_string());
    }
}

#[tokio::test]
async fn failed_command_does_not_poison_the_session() {
    let mut conn = connect();
    assert!(conn.execute_command("exit 1").await.is_err());
    let out = conn.execute_command("echo still-alive").await.unwrap();
    assert_eq!(out.stdout.trim(), "still-alive");
}

#[tokio::test]
async fn large_output_is_fully_captured() {
    let mut conn = connect();
    let out = conn
        .execute_command("head -c 1048576 /dev/zero | tr '\\0' 'a'")
        .await
        .unwrap();
    assert_eq!(out.stdout.len(), 1_048_576);
    assert!(out.stdout.bytes().all(|b| b == b'a'));
}

#[tokio::test]
async fn heredoc_file_write_roundtrips() {
    let mut conn = connect();
    let body = "FROM debian:stable\nRUN echo \"hi $HOME\"\nCMD [\"sh\"]";
    conn.execute_command(&format!(
        "cat > /tmp/ssh_test_heredoc <<'PIVX_EOF'\n{body}\nPIVX_EOF"
    ))
    .await
    .unwrap();

    let out = conn
        .execute_command("cat /tmp/ssh_test_heredoc")
        .await
        .unwrap();
    assert_eq!(out.stdout, format!("{body}\n"));

    conn.execute_command("rm -f /tmp/ssh_test_heredoc")
        .await
        .unwrap();
}

#[tokio::test]
async fn reads_os_release() {
    let mut conn = connect();
    let out = conn.execute_command("cat /etc/os-release").await.unwrap();
    assert!(out.stdout.contains("ID="), "got: {}", out.stdout);
}

#[tokio::test]
async fn escape_string_roundtrips_through_a_real_shell() {
    let mut conn = connect();
    for original in ["plain", "it's quoted", "back\\slash", "multi\nline"] {
        let cmd = format!("printf '%s' '{}'", escape_string(original));
        let out = conn.execute_command(&cmd).await.unwrap();
        assert_eq!(out.stdout, original, "roundtrip failed for {original:?}");
    }
}

#[tokio::test]
async fn wrong_username_fails_authentication() {
    let mut conn = SshConnection::from_private_key(
        "no_such_user_xyz".into(),
        key_path(),
        (host(), port()),
    );
    assert!(conn.execute_command("true").await.is_err());
}

#[tokio::test]
async fn missing_private_key_file_is_an_error() {
    let mut conn = SshConnection::from_private_key(
        user(),
        PathBuf::from("/nonexistent/id_ed25519"),
        (host(), port()),
    );
    assert!(conn.execute_command("true").await.is_err());
}

#[tokio::test]
async fn wrong_password_cannot_run_commands() {
    let mut conn = SshConnection::from_username_password(
        user(),
        "definitely-the-wrong-password".into(),
        (host(), port()),
    );
    assert!(conn.execute_command("echo should-not-run").await.is_err());
}

#[tokio::test]
async fn connection_refused_is_an_error() {
    let mut conn = SshConnection::from_private_key(
        user(),
        key_path(),
        ("127.0.0.1".to_string(), 1u16),
    );
    assert!(conn.execute_command("true").await.is_err());
}
