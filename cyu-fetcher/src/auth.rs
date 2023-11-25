use itertools::Itertools;
use regex::Regex;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;

lazy_static::lazy_static! {
    static ref SECURITY_TOKEN_REGEX: Regex = Regex::new(r#"<input name="__RequestVerificationToken" type="hidden" value="([^"]+)" />"#).unwrap();
    static ref FEDERATION_ID_REGEX: Regex = Regex::new(r#"var federationIdStr = '(.*?)';"#).unwrap();
}

pub async fn login(
    requester: &reqwest::Client,
    username: String,
    password: String,
) -> Result<String, ()> {
    let page_response = requester
        .get("https://services-web.cyu.fr/calendar/LdapLogin")
        .send()
        .await
        .map_err(|_| ())?;
    let page_cookie = page_response
        .headers()
        .get("set-cookie")
        .map(|cookie| cookie.clone())
        .ok_or(())?;
    let plain_text = page_response.text().await.map_err(|_| ())?;
    let token = SECURITY_TOKEN_REGEX
        .captures(&plain_text)
        .and_then(|captures| captures.get(1))
        .map(|token| token.as_str().to_string())
        .ok_or(())?;

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
        .map_err(|_| ())?;
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

pub async fn get_infos(requester: &reqwest::Client, token: String) -> Result<InfosResponse, ()> {
    let federation_id_response = requester
        .get("https://services-web.cyu.fr/calendar")
        .header("Cookie", token.clone())
        .send()
        .await
        .map_err(|_| ())?;

    let plain_text = federation_id_response.text().await.map_err(|_| ())?;

    let federation_id = FEDERATION_ID_REGEX
        .captures(&plain_text)
        .and_then(|captures| captures.get(1))
        .map(|federation_id| federation_id.as_str().to_string())
        .ok_or(())?;

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
        .map_err(|_| ())?;

    let infos = name_response
        .json::<Vec<InfosResponse>>()
        .await
        .map_err(|_| ())
        .and_then(|infos| infos.into_iter().next().ok_or(()))?;

    Ok(infos)
}
