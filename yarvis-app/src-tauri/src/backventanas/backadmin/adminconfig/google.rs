// ============================================================
// google.rs — Inicio de sesión con Google (OAuth 2.0 + PKCE)
//
// Flujo estilo "installed app / loopback" (parecido a Antigravity):
//   1. Genera verifier + challenge PKCE S256.
//   2. Abre el navegador en accounts.google.com (prompt select_account).
//   3. Google redirige a http://127.0.0.1:PUERTO/callback?code=...
//   4. Intercambia el code por tokens (cliente público: sin secret).
//   5. Pide userinfo (nombre + email).
//
// Para activarlo: define la variable YARVIS_GOOGLE_CLIENT_ID con el
// Client ID de una app "Desktop" de Google Cloud Console
// (https://console.cloud.google.com/apis/credentials).
// Sin ella, devuelve un perfil SIMULADO para probar la UI (demo).
// ============================================================

use rand::RngCore;
use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri_plugin_opener::OpenerExt;

const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const USERINFO_URL: &str = "https://openidconnect.googleapis.com/v1/userinfo";

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PerfilGoogle {
    pub nombre: String,
    pub email: String,
    pub simulado: bool,
}

fn client_id() -> Option<String> {
    std::env::var("YARVIS_GOOGLE_CLIENT_ID")
        .ok()
        .filter(|s| !s.is_empty())
}

fn b64url(data: &[u8]) -> String {
    use base64::engine::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
}

/// Inicia sesión con Google. Sin CLIENT_ID configurado regresa un perfil
/// simulado (`simulado: true`) para poder probar el flujo en la UI.
#[tauri::command]
pub async fn login_con_google(app: tauri::AppHandle) -> Result<PerfilGoogle, String> {
    let Some(cid) = client_id() else {
        return Ok(PerfilGoogle {
            nombre: String::new(),
            email: String::new(),
            simulado: true,
        });
    };

    // 1) PKCE S256
    let mut verif = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut verif);
    let verifier = b64url(&verif);
    let challenge = b64url(&Sha256::digest(verifier.as_bytes()));

    // 2) Servidor loopback en un puerto libre
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .map_err(|e| format!("No se pudo abrir el puerto local: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("Puerto inválido: {e}"))?
        .port();
    let redirect_uri = format!("http://localhost:{port}/callback");

    let mut url = url::Url::parse(AUTH_URL).map_err(|e| e.to_string())?;
    {
        let mut q = url.query_pairs_mut();
        q.append_pair("client_id", &cid);
        q.append_pair("redirect_uri", &redirect_uri);
        q.append_pair("response_type", "code");
        q.append_pair("scope", "openid email profile");
        q.append_pair("code_challenge", &challenge);
        q.append_pair("code_challenge_method", "S256");
        q.append_pair("prompt", "select_account");
    }

    // 3) Abrir el navegador con la pantalla de Google
    app.opener()
        .open_url(url.as_str(), None::<&str>)
        .map_err(|e| format!("No se pudo abrir el navegador: {e}"))?;

    // 4) Esperar el callback con el code
    let code = esperar_callback(listener).await?;

    // 5) Intercambiar code por tokens (cliente público, sin secret)
    let client = reqwest::Client::new();
    let token_resp: serde_json::Value = client
        .post(TOKEN_URL)
        .form(&[
            ("code", code.as_str()),
            ("client_id", cid.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
            ("code_verifier", verifier.as_str()),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|e| format!("Error pidiendo el token: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Respuesta de token inválida: {e}"))?;

    let access = token_resp
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("Google no devolvió access_token: {token_resp}"))?;

    // 6) Datos del perfil (nombre + email)
    let perfil: serde_json::Value = client
        .get(USERINFO_URL)
        .bearer_auth(access)
        .send()
        .await
        .map_err(|e| format!("Error pidiendo el perfil: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Perfil inválido: {e}"))?;

    let nombre = perfil
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let email = perfil
        .get("email")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    Ok(PerfilGoogle {
        nombre,
        email,
        simulado: false,
    })
}

/// Espera (máx. 120 s) a que Google redirija al loopback y extrae el `code`.
async fn esperar_callback(listener: tokio::net::TcpListener) -> Result<String, String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let fut = async {
        let (mut stream, _) = listener
            .accept()
            .await
            .map_err(|e| format!("Error en el loopback: {e}"))?;
        let mut buf = vec![0u8; 8192];
        let n = stream
            .read(&mut buf)
            .await
            .map_err(|e| format!("Error leyendo el callback: {e}"))?;
        let request = String::from_utf8_lossy(&buf[..n]).to_string();
        let code = extraer_code(&request)
            .ok_or_else(|| "El callback de Google no trajo `code`".to_string())?;

        let body = "<!doctype html><html><body style='font-family:sans-serif;text-align:center;padding-top:4rem'><h2>¡Ya puedes cerrar esta pestaña!</h2><p>Vuelve a Y.A.R.V.I.S.</p></body></html>";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = stream.write_all(response.as_bytes()).await;
        Ok::<String, String>(code)
    };

    tokio::time::timeout(std::time::Duration::from_secs(120), fut)
        .await
        .map_err(|_| "Se agotó el tiempo esperando el login de Google".to_string())?
}

fn extraer_code(request: &str) -> Option<String> {
    let uri = request.lines().next()?.split_whitespace().nth(1)?;
    let (_, query) = uri.split_once('?')?;
    for par in query.split('&') {
        if let Some(v) = par.strip_prefix("code=") {
            return Some(v.to_string());
        }
    }
    None
}
