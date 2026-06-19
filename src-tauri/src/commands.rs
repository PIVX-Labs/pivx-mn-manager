use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::net::ToSocketAddrs;

use crate::ssh::SshConnection;

#[tauri::command]
async fn init_vps_with_password(ip_address: &str, username: &str, password: &str) -> Result<()> {
    Ok(init_vps_internal(SshConnection::from_username_password(
        username.to_string(),
        password.to_string(),
        ip_address,
    ))
    .await?)
}

#[tauri::command]
async fn init_vps_with_key(ip_address: &str, username: &str, private_key_path: &str) -> Result<()> {
    let buf = PathBuf::try_from(private_key_path).with_context(|| "Invalid private_key_path")?;
    Ok(init_vps_internal(SshConnection::from_private_key(
        username.to_string(),
        buf,
        ip_address,
    ))
    .await?)
}

async fn init_vps_internal<T: ToSocketAddrs>(mut ssh_connection: SshConnection<T>) -> Result<()> {
    ssh_connection.download_dependencies().await?;
    ssh_connection.setup_pivxd_docker().await?;
    Ok(())
}
