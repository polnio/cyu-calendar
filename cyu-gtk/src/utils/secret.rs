use super::auth::Auth;
use super::constants::APP_ID;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Mutex;

pub static SECRET: Lazy<Secret> = Lazy::new(|| Secret::new());

struct LibsecretSchema(libsecret::Schema);
unsafe impl Send for LibsecretSchema {}
// unsafe impl Sync for LibsecretSchema {}
impl From<libsecret::Schema> for LibsecretSchema {
    fn from(schema: libsecret::Schema) -> Self {
        Self(schema)
    }
}
impl Deref for LibsecretSchema {
    type Target = libsecret::Schema;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Secret {
    schema: Mutex<LibsecretSchema>,
}

impl Secret {
    pub fn new() -> Self {
        let attributes = HashMap::from([("name", libsecret::SchemaAttributeType::String)]);
        let schema = libsecret::Schema::new(APP_ID, libsecret::SchemaFlags::NONE, attributes);
        Self {
            schema: Mutex::new(schema.into()),
        }
    }

    async fn get(&self, key: &str) -> Option<String> {
        let schema = self.schema.lock().unwrap();
        let attributes = HashMap::from([("name", key)]);
        let value = libsecret::password_lookup_future(Some(&schema), attributes).await;
        match value {
            Ok(value) => value.map(|value| value.to_string()),
            Err(_) => {
                println!("Failed to get secret");
                None
            }
        }
    }
    async fn set(&self, key: &str, value: String) {
        let schema = self.schema.lock().unwrap();
        let attributes = HashMap::from([("name", key)]);
        let result = libsecret::password_store_future(
            Some(&schema),
            attributes,
            Some(libsecret::COLLECTION_DEFAULT),
            key,
            value.as_str(),
        )
        .await;
        if result.is_err() {
            println!("Failed to store secret");
        }
    }

    async fn remove(&self, key: &str) {
        let schema = self.schema.lock().unwrap();
        let attributes = HashMap::from([("name", key)]);
        let result = libsecret::password_clear_future(Some(&schema), attributes).await;
        if result.is_err() {
            println!("Failed to clear secret");
        }
    }

    pub async fn get_auth(&self) -> Option<Auth> {
        let token = self.get("token").await;
        let id = self.get("id").await;
        let name = self.get("name").await;
        token
            .zip(id)
            .zip(name)
            .map(|((token, id), name)| Auth { token, id, name })
    }

    pub async fn set_auth(&self, auth: Auth) {
        self.set("token", auth.token).await;
        self.set("id", auth.id).await;
        self.set("name", auth.name).await;
    }

    pub async fn remove_auth(&self) {
        self.remove("token").await;
        self.remove("id").await;
        self.remove("name").await;
    }
}
