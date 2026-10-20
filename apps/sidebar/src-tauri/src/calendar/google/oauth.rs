//! OAuth 2.0 for installed apps: system browser, loopback redirect and PKCE
//! (RFC 8252, RFC 7636). No embedded web view ever sees the Google password.
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use reqwest::blocking::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    io::{ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    time::{Duration, Instant},
};

const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const REVOKE_URL: &str = "https://oauth2.googleapis.com/revoke";
/// Least privilege: list calendars, edit events. No calendar settings,
/// sharing or deletion of whole calendars.
pub const SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/calendar.calendarlist.readonly",
    "https://www.googleapis.com/auth/calendar.events",
];
const LOGIN_TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub scope: String,
}

#[derive(Deserialize)]
struct TokenError {
    error: String,
    error_description: Option<String>,
}

pub enum RefreshError {
    /// Refresh token revoked or expired. The user must connect again.
    Revoked,
    Other(String),
}

fn random_token() -> Result<String, String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|e| e.to_string())?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

pub fn challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

/// Opens the consent page and waits for the loopback redirect.
pub fn authorize(
    http: &Client,
    client_id: &str,
    client_secret: &str,
    open: impl FnOnce(&str) -> Result<(), String>,
) -> Result<TokenResponse, String> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let redirect = format!(
        "http://127.0.0.1:{}/callback",
        listener.local_addr().map_err(|e| e.to_string())?.port()
    );
    let verifier = random_token()?;
    let state = random_token()?;
    let scope = SCOPES.join(" ");
    let url = reqwest::Url::parse_with_params(
        AUTH_URL,
        [
            ("client_id", client_id),
            ("redirect_uri", &redirect),
            ("response_type", "code"),
            ("scope", &scope),
            ("code_challenge", &challenge(&verifier)),
            ("code_challenge_method", "S256"),
            ("state", &state),
            ("access_type", "offline"),
            // Always issue a refresh token, also when reconnecting.
            ("prompt", "consent"),
        ],
    )
    .map_err(|e| e.to_string())?;
    open(url.as_str())?;
    let code = wait_for_code(&listener, &state)?;
    let token: TokenResponse = token_request(
        http,
        &[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", &redirect),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("code_verifier", &verifier),
        ],
    )
    .map_err(|e| match e {
        RefreshError::Revoked => "Google hat die Anmeldung abgelehnt. Erneut verbinden.".into(),
        RefreshError::Other(e) => e,
    })?;
    // Google lets users untick individual scopes on the consent page.
    if SCOPES
        .iter()
        .any(|s| !token.scope.split(' ').any(|g| g == *s))
    {
        return Err(
            "Nicht alle Berechtigungen erteilt. Beim Verbinden beide Kalender-Häkchen setzen."
                .into(),
        );
    }
    if token.refresh_token.is_none() {
        return Err("Google hat kein Refresh-Token geliefert. Erneut verbinden.".into());
    }
    Ok(token)
}

pub fn refresh(
    http: &Client,
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<TokenResponse, RefreshError> {
    token_request(
        http,
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", client_id),
            ("client_secret", client_secret),
        ],
    )
}

/// Best effort. Local credentials are removed regardless of the outcome.
pub fn revoke(http: &Client, token: &str) {
    let _ = http.post(REVOKE_URL).form(&[("token", token)]).send();
}

fn token_request(http: &Client, form: &[(&str, &str)]) -> Result<TokenResponse, RefreshError> {
    let response = http
        .post(TOKEN_URL)
        .form(form)
        .send()
        .map_err(|e| RefreshError::Other(format!("Google nicht erreichbar: {e}")))?;
    if response.status().is_success() {
        return response
            .json()
            .map_err(|e| RefreshError::Other(e.to_string()));
    }
    let error: TokenError = response
        .json()
        .map_err(|e| RefreshError::Other(e.to_string()))?;
    if error.error == "invalid_grant" {
        return Err(RefreshError::Revoked);
    }
    Err(RefreshError::Other(format!(
        "Google-Anmeldung fehlgeschlagen: {}",
        error.error_description.unwrap_or(error.error)
    )))
}

fn wait_for_code(listener: &TcpListener, state: &str) -> Result<String, String> {
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let deadline = Instant::now() + LOGIN_TIMEOUT;
    while Instant::now() < deadline {
        let mut stream = match listener.accept() {
            Ok((stream, _)) => stream,
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(200));
                continue;
            }
            Err(e) => return Err(e.to_string()),
        };
        // Browsers also request /favicon.ico; only the callback counts.
        let Some(params) = read_callback(&mut stream) else {
            respond(&mut stream, "404 Not Found", "");
            continue;
        };
        let get = |key: &str| {
            params
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
        };
        if get("state").as_deref() != Some(state) {
            respond(
                &mut stream,
                "400 Bad Request",
                "Ungültige Anmeldung. Fenster schließen und erneut verbinden.",
            );
            continue;
        }
        if let Some(error) = get("error") {
            respond(
                &mut stream,
                "200 OK",
                "Anmeldung abgebrochen. Du kannst dieses Fenster schließen.",
            );
            return Err(if error == "access_denied" {
                "Google-Anmeldung abgebrochen.".into()
            } else {
                format!("Google-Anmeldung fehlgeschlagen: {error}")
            });
        }
        if let Some(code) = get("code") {
            respond(
                &mut stream,
                "200 OK",
                "todoSideBar ist mit Google Kalender verbunden. Du kannst dieses Fenster schließen.",
            );
            return Ok(code);
        }
        respond(
            &mut stream,
            "400 Bad Request",
            "Antwort von Google unvollständig.",
        );
    }
    Err("Google-Anmeldung hat zu lange gedauert. Erneut verbinden.".into())
}

fn read_callback(stream: &mut TcpStream) -> Option<Vec<(String, String)>> {
    stream.set_nonblocking(false).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok()?;
    let mut buffer = [0u8; 8192];
    let mut length = 0;
    while length < buffer.len() {
        let read = stream.read(&mut buffer[length..]).ok()?;
        if read == 0 {
            break;
        }
        length += read;
        if buffer[..length].windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
    }
    let request = std::str::from_utf8(&buffer[..length]).ok()?;
    let target = request
        .lines()
        .next()?
        .strip_prefix("GET ")?
        .split(' ')
        .next()?;
    let url = reqwest::Url::parse(&format!("http://127.0.0.1{target}")).ok()?;
    (url.path() == "/callback").then(|| url.query_pairs().into_owned().collect())
}

fn respond(stream: &mut TcpStream, status: &str, message: &str) {
    let body = format!(
        "<!doctype html><meta charset=utf-8><title>todoSideBar</title>\
         <body style=\"font:15px system-ui;margin:3rem\"><p>{message}</p>"
    );
    let _ = write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_challenge_matches_rfc_7636_example() {
        assert_eq!(
            challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn callback_ignores_other_paths() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        for (path, expected) in [
            ("/favicon.ico", None),
            (
                "/callback?code=a%2Fb&state=s",
                Some(vec![("code", "a/b"), ("state", "s")]),
            ),
        ] {
            let mut client = TcpStream::connect(addr).unwrap();
            write!(client, "GET {path} HTTP/1.1\r\nHost: x\r\n\r\n").unwrap();
            let (mut server, _) = listener.accept().unwrap();
            let params = read_callback(&mut server);
            let expected = expected.map(|pairs| {
                pairs
                    .into_iter()
                    .map(|(k, v)| (k.to_owned(), v.to_owned()))
                    .collect::<Vec<_>>()
            });
            assert_eq!(params, expected);
        }
    }
}
