use crate::errors::Error;
use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;

pub async fn login(
    requester: &reqwest::Client,
    username: String,
    password: String,
) -> Result<String, Error> {
    let page_response = requester
        .get("https://services-web.cyu.fr/calendar/LdapLogin")
        .send()
        .await
        .map_err(|_| Error::Remote)?;
    let page_cookie = page_response
        .headers()
        .get("set-cookie")
        .map(|cookie| cookie.clone())
        .ok_or(Error::Remote)?;
    let plain_text = page_response.text().await.map_err(|_| Error::Remote)?;

    static SECURITY_TOKEN_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"<input name="__RequestVerificationToken" type="hidden" value="([^"]+)" />"#)
            .unwrap()
    });
    let token = SECURITY_TOKEN_REGEX
        .captures(&plain_text)
        .and_then(|captures| captures.get(1))
        .map(|token| token.as_str().to_string())
        .ok_or(Error::Remote)?;

    let mut remote_payload = HashMap::new();
    remote_payload.insert("Name", username);
    remote_payload.insert("Password", password);
    remote_payload.insert("__RequestVerificationToken", token);

    let login_response = requester
        .post("https://services-web.cyu.fr/calendar/LdapLogin/Logon")
        .form(&remote_payload)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Cookie", page_cookie)
        .send()
        .await
        .map_err(|_| Error::Remote)?;

    if !login_response.status().is_redirection() {
        return Err(Error::Unauthorized);
    }

    let login_cookie = login_response
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|cookie| cookie.to_str().unwrap_or_default().to_string())
        .join(";");

    Ok(login_cookie)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InfosResponse {
    pub federation_id: String,
    pub display_name: String,
}

pub async fn get_infos(requester: &reqwest::Client, token: String) -> Result<InfosResponse, Error> {
    let federation_id_response = requester
        .get("https://services-web.cyu.fr/calendar")
        .header("Cookie", token.clone())
        .send()
        .await
        .map_err(|_| Error::Remote)?;

    let plain_text = federation_id_response
        .text()
        .await
        .map_err(|_| Error::Remote)?;

    static FEDERATION_ID_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"var federationIdStr = '(.*?)';"#).unwrap());
    let federation_id = FEDERATION_ID_REGEX
        .captures(&plain_text)
        .and_then(|captures| captures.get(1))
        .map(|federation_id| federation_id.as_str().to_string())
        .ok_or(Error::Unauthorized)?;

    let name_response = requester
        .post("https://services-web.cyu.fr/calendar/Home/LoadDisplayNames")
        .form(&json!({
            "federationIds[]": federation_id,
            "resType": 104
        }))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Cookie", token)
        .send()
        .await
        .map_err(|_| Error::Remote)?;

    let infos = name_response
        .json::<Vec<InfosResponse>>()
        .await
        .map_err(|_| Error::Remote)
        .and_then(|infos| infos.into_iter().next().ok_or(Error::Remote))?;

    Ok(infos)
}
