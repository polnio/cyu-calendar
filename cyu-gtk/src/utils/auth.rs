use crate::utils::{config::CONFIG, secret::SECRET, FETCHER};
use std::sync::RwLock;

pub const AUTH: RwLock<Option<Auth>> = RwLock::new(None);

pub enum Error {
    Remote,
    BadCredentials,
}

#[derive(Debug, Clone)]
pub struct Auth {
    pub token: String,
    pub id: String,
    pub name: String,
    pub username: String,
    pub password: String,
}

pub async fn init() {
    if CONFIG.read().unwrap().save_credentials() {
        *AUTH.write().unwrap() = SECRET.get_auth().await;
    } else {
        SECRET.remove_auth().await;
        AUTH.write().unwrap().take();
    }
}

pub async fn login(username: String, password: String) -> Result<(), Error> {
    let token = FETCHER
        .login(username.clone().into(), password.clone().into())
        .await
        .map_err(|_| Error::Remote)?;
    let infos = FETCHER
        .get_infos(token.clone())
        .await
        .map_err(|_| Error::BadCredentials)?;
    let auth = Auth {
        token,
        id: infos.federation_id,
        name: infos.display_name,
        username,
        password,
    };
    if CONFIG.read().unwrap().save_credentials() {
        SECRET.set_auth(auth.clone()).await;
    }
    AUTH.write().unwrap().replace(auth);

    Ok(())
}

pub async fn logout() {
    SECRET.remove_auth().await;
    {
        AUTH.write().unwrap().take();
    }
}

pub async fn refetch() -> Result<(), Error> {
    let auth = SECRET.get_auth().await.ok_or(Error::BadCredentials)?;
    login(auth.username, auth.password).await
}
