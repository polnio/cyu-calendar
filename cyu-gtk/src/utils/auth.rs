use crate::utils::{secret::SECRET, FETCHER};
use std::sync::RwLock;

pub static AUTH: RwLock<Option<Auth>> = RwLock::new(None);

pub enum Error {
    Remote,
    BadCredentials,
}

#[derive(Debug, Clone)]
pub struct Auth {
    pub token: String,
    pub id: String,
    pub name: String,
}

pub async fn init() {
    let auth = SECRET.get_auth().await;
    {
        *AUTH.write().unwrap() = auth;
    }
}

pub async fn login(username: String, password: String) -> Result<(), Error> {
    let token = FETCHER
        .login(username.into(), password.into())
        .await
        .map_err(|_| Error::Remote)?;
    let infos = FETCHER
        .get_infos(token.clone())
        .await
        .map_err(|_| Error::BadCredentials)?;
    println!("Logged in : {}", infos.display_name);
    let auth = Auth {
        token,
        id: infos.federation_id,
        name: infos.display_name,
    };
    SECRET.set_auth(auth.clone()).await;
    {
        AUTH.write().unwrap().replace(auth);
    }
    Ok(())
}

pub async fn logout() {
    SECRET.remove_auth().await;
    {
        AUTH.write().unwrap().take();
    }
}
