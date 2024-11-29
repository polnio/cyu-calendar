use super::Auth;
use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use anyhow::{anyhow, bail, Context as _, Result};
use base64::prelude::*;
use cyu_fetcher::calendar::ColorBy;
use cyu_fetcher::Fetcher;
use icalendar::{Component as _, EventLike as _};
use std::borrow::Cow;

pub async fn generate(fetcher: &Fetcher, auth: Auth) -> Result<String> {
    let events = fetcher
        .get_all_calendar(cyu_fetcher::calendar::GetAllQuery {
            id: auth.id,
            token: auth.token,
            color_by: ColorBy::EventCategory,
        })
        .await
        // .map_err(|_| Error::RemoteError)?;
        .context("Failed to get all calendar")?;

    let events = events.into_iter().filter_map(|event| {
        let mut ievent = icalendar::Event::new();

        let description = event.description();

        ievent
            .uid(&format!("{}@cyu-calendar", event.id()))
            .description(&description);

        match (event.all_day(), event.end()) {
            (true, _) => ievent.all_day((**event.start()).into()),
            (false, Some(end)) => ievent
                .starts::<chrono::NaiveDateTime>(**event.start())
                .ends::<chrono::NaiveDateTime>(**end),
            (false, None) => return None,
        };

        let category = event.event_category();
        let title: Cow<str> = match category.as_str() {
            // "CM" => format!("CM {}", event.description().split('\n').rev().nth(2).unwrap_or_default()).into(),
            // "TD" => description.split('\n').rev().nth(2).unwrap_or_default().into(),
            "CM" | "TD" => format!(
                "{} {}",
                category,
                description
                    .split('\n')
                    .rev()
                    .nth(2)
                    .unwrap_or_default()
                    .replace(category, "")
            )
            .into(),
            cat => cat.into(),
        };
        ievent.summary(&title);

        Some(ievent.done())
    });

    let mut calendar = icalendar::Calendar::from_iter(events);
    let calendar = calendar.name("CYU Calendar");

    Ok(calendar.to_string().replace("\\N", "\\n"))
}

pub type Encrypter = Aes256Gcm;

pub fn encrypt_creds(encrypter: &Encrypter, username: &str, password: &str) -> Result<String> {
    let to_encrypt = format!("{}:{}", username, password);
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    let encrypted = encrypter
        .encrypt(Nonce::from_slice(&nonce), to_encrypt.as_bytes())
        .map_err(|err| anyhow!("{}", err))
        .context("")?;
    let mut result = nonce.to_vec();
    result.extend_from_slice(&encrypted);
    Ok(BASE64_URL_SAFE_NO_PAD.encode(result))
}

pub fn decrypt_creds(encrypter: &Encrypter, encrypted: &str) -> Result<(String, String)> {
    let data = BASE64_URL_SAFE_NO_PAD
        .decode(encrypted)
        .context("Failed to decode base64")?;
    if data.len() < 12 {
        bail!("Invalid token");
    }
    let nonce = Nonce::from_slice(&data[..12]);
    let ciphertext = &data[12..];
    let decrypted = encrypter
        .decrypt(nonce, ciphertext)
        .map_err(|err| anyhow!("{}", err))
        .context("Failed to decrypt token")?;
    let decrypted = String::from_utf8_lossy(&decrypted);
    let (username, password) = decrypted.split_once(':').context("Invalid token")?;
    Ok((username.to_owned(), password.to_owned()))
}
