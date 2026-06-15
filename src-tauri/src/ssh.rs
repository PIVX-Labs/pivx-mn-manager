use crate::distro::select_distro_etc_release;
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

    pub fn from_private_key(username: String, private_key_path: PathBuf) -> Self {
        todo!()
    }

    async fn get_session_or_connect(&mut self) -> Result<&mut Handle<Client>, russh::Error> {
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
                    return Err(russh::Error::NotAuthenticated);
                }
            } else if let Some(password) = &self.password {
                session
                    .authenticate_password(&self.username, password)
                    .await?;
            }
            self.session = Some(session);
        }
        // We've either just assiged session, or it was Some to beign with
        Ok(self.session.as_mut().unwrap())
    }

    async fn execute_command(&mut self, command: &str) -> Result<String, russh::Error> {
        let handle = self.get_session_or_connect().await?;
        let mut channel = handle.channel_open_session().await?;
        channel.exec(true, command).await?;
        let mut response = String::new();

        loop {
            let Some(msg) = channel.wait().await else {
                break;
            };
            match msg {
                ChannelMsg::Data { ref data } => {
                    response.push_str(str::from_utf8(data).unwrap_or(""));
                }
                _ => {}
            }
        }
        Ok(response)
    }

    pub async fn requires_sudo_password(&self) -> Result<bool, russh::Error> {
        todo!()
    }

    pub async fn set_sudo_passowrd(&mut self, sudo_password: String) -> Result<(), russh::Error> {
        self.sudo_password = Some(sudo_password);
        // TODO: check that it's correct
        Ok(())
    }

    pub async fn download_dependencies(&mut self) -> Result<(), russh::Error> {
        let distro = select_distro_etc_release(&self.execute_command("cat /etc/os-release").await?);
        if let Some(distro) = distro {
            self.execute_command(&distro.install_deps_command()).await?;

            Ok(())
        } else {
            Err(russh::Error::UnsupportedAuthMethod)
        }
    }

    pub async fn setup_pivxd_docker(&self) -> Result<(), russh::Error> {
        todo!()
    }
}

struct Client {}

impl client::Handler for Client {
    type Error = russh::Error;
}
