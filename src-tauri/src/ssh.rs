use crate::distro::select_distro_etc_release;
use anyhow::{Context, Result};
use russh::client::{self, Handle};
use russh::keys::PrivateKeyWithHashAlg;
use russh::ChannelMsg;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::ToSocketAddrs;

pub struct SshConnection<T: ToSocketAddrs> {
    sudo_password: Option<String>,
    username: String,
    password: Option<String>,
    private_key_path: Option<PathBuf>,
    address: T,
    session: Option<Handle<Client>>,
}

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
}

impl<T: ToSocketAddrs> SshConnection<T> {
    pub fn from_username_password(
        username: String,
        password: String,
        address: T,
    ) -> SshConnection<T> {
        Self {
            username,
            password: Some(password),
            sudo_password: None,
            private_key_path: None,
            address: address,
            session: None,
        }
    }

    pub fn from_private_key(
        username: String,
        private_key_path: PathBuf,
        address: T,
    ) -> SshConnection<T> {
        Self {
            username,
            password: None,
            sudo_password: None,
            private_key_path: Some(private_key_path),
            address,
            session: None,
        }
    }

    async fn get_session_or_connect(&mut self) -> Result<&mut Handle<Client>> {
        if self.session.is_none() {
            let config = Arc::new(client::Config {
                ..Default::default()
            });

            let mut session = client::connect(config, &self.address, Client {}).await?;
            if let Some(private_key_path) = &self.private_key_path {
                let key_pair = russh::keys::load_secret_key(private_key_path, None)?;
                let auth_res = session
                    .authenticate_publickey(
                        &self.username,
                        PrivateKeyWithHashAlg::new(
                            Arc::new(key_pair),
                            session.best_supported_rsa_hash().await?.flatten(),
                        ),
                    )
                    .await?;
                if !auth_res.success() {
                    return Err(russh::Error::NotAuthenticated)
                        .with_context(|| "Authentication with private key failed");
                }
            } else if let Some(password) = &self.password {
                session
                    .authenticate_password(&self.username, password)
                    .await
                    .with_context(|| "Authentication with password failed")?;
            }
            self.session = Some(session);
        }
        // We've either just assiged session, or it was Some to beign with
        Ok(self.session.as_mut().unwrap())
    }

    async fn execute_command(&mut self, command: &str) -> Result<CommandOutput> {
        let handle = self.get_session_or_connect().await?;
        let mut channel = handle.channel_open_session().await?;
        channel
            .exec(true, command)
            .await
            .with_context(|| format!("Failed to execute command {command}"))?;

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut exit_status = None;

        while let Some(msg) = channel.wait().await {
            match msg {
                ChannelMsg::Data { ref data } => stdout.extend_from_slice(data),
                ChannelMsg::ExtendedData { ref data, .. } => stderr.extend_from_slice(data),
                ChannelMsg::ExitStatus { exit_status: s } => exit_status = Some(s),
                _ => {}
            }
        }

        let stdout = String::from_utf8_lossy(&stdout).into_owned();
        let stderr = String::from_utf8_lossy(&stderr).into_owned();

        match exit_status {
            Some(0) => Ok(CommandOutput { stdout, stderr }),
            Some(code) => Err(anyhow::anyhow!(
                "command `{command}` exited with {code}: {stderr}"
            )),
            None => Err(anyhow::anyhow!(
                "command `{command}` did not report an exit status"
            )),
        }
    }
    pub async fn requires_sudo_password(&mut self) -> Result<bool> {
        let result = self.execute_command("sudo echo hello").await?;
        if let Some(password) = self.password.take() {
            self.execute_command(&password).await?;
            self.password = Some(password);
        }
        Ok(result.stdout.trim() == "hello")
    }

    pub async fn set_sudo_passowrd(&mut self, sudo_password: String) -> Result<bool> {
        self.sudo_password = Some(sudo_password);
        Ok(self.requires_sudo_password().await?)
    }

    pub async fn download_dependencies(&mut self) -> Result<()> {
        let distro =
            select_distro_etc_release(&self.execute_command("cat /etc/os-release").await?.stdout);
        if let Some(distro) = distro {
            self.execute_command(&distro.install_deps_command()).await?;

            Ok(())
        } else {
            Err(russh::Error::UnsupportedAuthMethod.into())
        }
    }

    pub async fn setup_pivxd_docker(&mut self) -> Result<()> {
        let docker_file = include_str!("../../docker/Dockerfile");
        self.execute_command("cd && mkdir .pivx-mn-manager && cd .pivx-mn-manager")
            .await
            .context("Failed to create .pivx-mn-manager directory")?;
        self.execute_command(&format!(
            "cat > Dockerfile <<'PIVX_EOF'\n{docker_file}\nPIVX_EOF"
        ))
        .await
        .context("Failed to create Dockerfile")?;
        self.execute_command("docker buildx build -t pivx-masternode -f Dockerfile")
            .await
            .context("Failed to build Dockerfile")?;
        self.execute_command(
            "docker run -d --restart unless-stopped --name pivx-masternode pivx-masternode",
        )
        .await
        .context("Failed to run docker")?;
        Ok(())
    }
}

fn escape_string(string: &str) -> String {
    string
        .chars()
        .fold(String::with_capacity(string.len()), |mut acc, c| {
            match c {
                '\\' => acc.push_str("\\\\"),
                '\'' => acc.push_str("'\''"),
                '\n' => acc.push_str("\\n"),
                _ => acc.push(c),
            }
            acc
        })
}

struct Client {}

impl client::Handler for Client {
    type Error = russh::Error;
}
