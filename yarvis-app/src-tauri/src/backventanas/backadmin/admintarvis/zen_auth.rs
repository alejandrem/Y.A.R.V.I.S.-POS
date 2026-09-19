// ============================================================
// zen_auth — Cuenta OpenCode del dueño vinculada a YARVIS (OAuth).
//
// Por qué existe: el free tier de Zen solo responde a sesiones de
// usuario creadas en sus apps ("can only be used from within
// OpenCode"); una `sk-` suelta siempre recibe 403 aunque sea válida.
// El CLI oficial entra con OAuth (client_id `app`, verificado en
// código abierto de terceros); este módulo hace LO MISMO pero con la
// cuenta del dueño: él inicia sesión en SU navegador, YARVIS solo
// guarda los tokens. Sin suplantar nada: sin su login no hay sesión.
//
// Flujo (installed app / loopback, como google.rs):
//   1. PKCE S256 + servidor loopback en puerto libre.
//   2. Se abre el navegador en auth.opencode.ai (él se loguea ahí).
//   3. El code vuelve a http://127.0.0.1:PUERTO/callback.
//   4. Se intercambia por access + refresh tokens (cliente público).
//   5. Se guardan en app_data_dir/zen_sesion.json con permisos 0600.
//   6. El chat usa el access token vigente (refresca solo si hace falta).
//
// Seguridad (igual que api_keys.json, ver issue #4):
//   - Solo ADMIN vincula/desvincula (los tokens gastan SU cuota).
//   - El frontend NUNCA ve tokens: el backend los sustituye al llamar.
//   - zen_estado expone solo {vinculado, email} (operario puede verlo).
// ============================================================

use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::Manager;
use tauri_plugin_opener::OpenerExt;

const AUTH_URL: &str = "https://auth.opencode.ai/authorize";
const TOKEN_URL: &str = "https://auth.opencode.ai/token";
const CLIENT_ID: &str = "app";

/// Sesión guardada en disco (0600). `email` es informativo (del JWT).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SesionZen {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expira_epoch: u64,
    pub email: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoZen {
    pub vinculado: bool,
    pub email: String,
}

fn ahora_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn b64url(data: &[u8]) -> String {
    use base64::engine::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
}

/// Par PKCE: (verifier para el intercambio, challenge para el authorize).
pub fn generar_pkce() -> (String, String) {
    let mut verif = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut verif);
    let verifier = b64url(&verif);
    let challenge = b64url(&Sha256::digest(verifier.as_bytes()));
    (verifier, challenge)
}

/// URL de login para abrir en el navegador del dueño.
pub fn url_autorizacion(puerto: u16, challenge: &str, estado: &str) -> String {
    format!(
        "{AUTH_URL}?client_id={CLIENT_ID}&redirect_uri=http://localhost:{puerto}/callback&response_type=code&code_challenge={challenge}&code_challenge_method=S256&state={estado}"
    )
}

/// Extrae el `code` del request HTTP crudo del loopback.
pub fn extraer_code(request: &str) -> Option<String> {
    let uri = request.lines().next()?.split_whitespace().nth(1)?;
    let (_, query) = uri.split_once('?')?;
    for par in query.split('&') {
        if let Some(v) = par.strip_prefix("code=") {
            return Some(v.to_string());
        }
    }
    None
}

/// Intenta leer el email del payload del JWT (access token de OpenAuth).
/// Puro y best-effort: si no es JWT o no trae email, cadena vacía.
pub fn email_de_jwt(token: &str) -> String {
    let partes: Vec<&str> = token.split('.').collect();
    if partes.len() != 3 {
        return String::new();
    }
    use base64::engine::Engine;
    let cuerpo = match base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(partes[1]) {
        Ok(b) => b,
        Err(_) => return String::new(),
    };
    serde_json::from_slice::<serde_json::Value>(&cuerpo)
        .ok()
        .and_then(|v| {
            v.get("email")
                .and_then(|e| e.as_str())
                .map(|e| e.to_string())
        })
        .unwrap_or_default()
}

/// ¿Sigue vigente? Con 5 min de margen para no morir a mitad de stream.
pub fn sesion_vigente(sesion: &SesionZen, ahora: u64) -> bool {
    !sesion.access_token.trim().is_empty() && ahora + 300 < sesion.expira_epoch
}

fn ruta_sesion(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|_| "No se pudo resolver el directorio de datos".to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("zen_sesion.json"))
}

fn escribir_0600(ruta: &std::path::Path, contenido: &str) -> Result<(), String> {
    std::fs::write(ruta, contenido).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(ruta, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn guardar_sesion_en(ruta: &std::path::Path, sesion: &SesionZen) -> Result<(), String> {
    let json = serde_json::to_string_pretty(sesion).map_err(|e| e.to_string())?;
    escribir_0600(ruta, &json)
}

pub fn cargar_sesion_de(ruta: &std::path::Path) -> Option<SesionZen> {
    let contenido = std::fs::read_to_string(ruta).ok()?;
    let sesion: SesionZen = serde_json::from_str(&contenido).ok()?;
    if sesion.access_token.trim().is_empty() {
        return None;
    }
    Some(sesion)
}

/// Intercambia el code del loopback por tokens (cliente público PKCE).
async fn intercambiar_code(
    code: &str,
    verifier: &str,
    redirect_uri: &str,
) -> Result<(String, Option<String>, u64), String> {
    let resp: serde_json::Value = reqwest::Client::new()
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("client_id", CLIENT_ID),
            ("code_verifier", verifier),
        ])
        .send()
        .await
        .map_err(|e| format!("Error pidiendo tokens a OpenCode: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Respuesta de tokens inválida: {e}"))?;
    extraer_tokens(&resp)
}

/// Refresca el access token con el refresh guardado.
async fn refrescar_sesion(
    refresh_token: &str,
) -> Result<(String, Option<String>, u64), String> {
    let resp: serde_json::Value = reqwest::Client::new()
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", CLIENT_ID),
        ])
        .send()
        .await
        .map_err(|e| format!("Error refrescando la sesión de OpenCode: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Respuesta de refresco inválida: {e}"))?;
    extraer_tokens(&resp)
}

/// Saca (access, refresh?, expira_epoch) de una respuesta de token.
/// Puro para probarlo sin red: OpenAuth manda `expires_in` en segundos.
pub fn extraer_tokens(resp: &serde_json::Value) -> Result<(String, Option<String>, u64), String> {
    let access = resp
        .get("access_token")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("OpenCode no devolvió access_token: {resp}"))?;
    let refresh = resp
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let expira = resp
        .get("expires_in")
        .and_then(|v| v.as_u64())
        .unwrap_or(3600);
    Ok((access.to_string(), refresh, ahora_epoch() + expira))
}

/// Espera (máx. 180 s) a que el dueño complete el login y extrae el `code`.
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
            .ok_or_else(|| "El callback de OpenCode no trajo `code`".to_string())?;

        let body = "<!doctype html><html><body style='font-family:sans-serif;text-align:center;padding-top:4rem'><h2>¡Cuenta vinculada!</h2><p>Vuelve a Y.A.R.V.I.S. y cierra esta pestaña.</p></body></html>";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = stream.write_all(response.as_bytes()).await;
        Ok::<String, String>(code)
    };

    tokio::time::timeout(std::time::Duration::from_secs(180), fut)
        .await
        .map_err(|_| "Se agotó el tiempo: completa el login en el navegador e inténtalo de nuevo".to_string())?
}

/// Access token vigente para llamar a Zen, refrescando si hace falta.
/// `None` = sin vincular (el chat usa la sk- del frente como siempre).
/// Nunca expone tokens al frontend: solo el string para el backend.
pub async fn token_sesion_vigente(app: &tauri::AppHandle) -> Option<String> {
    let ruta = ruta_sesion(app).ok()?;
    let mut sesion = cargar_sesion_de(&ruta)?;
    if sesion_vigente(&sesion, ahora_epoch()) {
        return Some(sesion.access_token);
    }
    let refresh = sesion.refresh_token.clone()?;
    let (access, nuevo_refresh, expira) = refrescar_sesion(&refresh).await.ok()?;
    sesion.access_token = access;
    if let Some(r) = nuevo_refresh {
        sesion.refresh_token = Some(r);
    }
    sesion.expira_epoch = expira;
    guardar_sesion_en(&ruta, &sesion).ok()?;
    Some(sesion.access_token)
}

/// Clave efectiva para el proveedor: si es Zen vinculado, el token de la
/// sesión del dueño; si no, la key que mandó el frontend (flujo actual).
pub async fn clave_para_proveedor(
    app: &tauri::AppHandle,
    proveedor: &str,
    api_key: String,
) -> String {
    if proveedor == "opencode" {
        if let Some(token) = token_sesion_vigente(app).await {
            return token;
        }
    }
    api_key
}

/// ¿Hay cuenta vinculada? Solo estado + email (sin secretos).
#[tauri::command]
pub async fn zen_estado(
    app: tauri::AppHandle,
    auth: tauri::State<'_, crate::backventanas::auth::AuthState>,
) -> Result<EstadoZen, String> {
    auth.require_operator()?;
    let ruta = ruta_sesion(&app)?;
    match cargar_sesion_de(&ruta) {
        Some(s) => Ok(EstadoZen { vinculado: true, email: s.email }),
        None => Ok(EstadoZen { vinculado: false, email: String::new() }),
    }
}

/// Vincula la cuenta OpenCode del dueño (SOLO admin: gasta su cuota).
/// Abre el navegador, espera su login, guarda la sesión y devuelve su email.
#[tauri::command]
pub async fn zen_login(
    app: tauri::AppHandle,
    auth: tauri::State<'_, crate::backventanas::auth::AuthState>,
) -> Result<String, String> {
    auth.require_admin()?;
    let (verifier, challenge) = generar_pkce();
    let estado: String = b64url(&{
        let mut b = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut b);
        b
    });

    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .map_err(|e| format!("No se pudo abrir el puerto local: {e}"))?;
    let puerto = listener
        .local_addr()
        .map_err(|e| format!("Puerto inválido: {e}"))?
        .port();
    let redirect_uri = format!("http://localhost:{puerto}/callback");

    app.opener()
        .open_url(url_autorizacion(puerto, &challenge, &estado).as_str(), None::<&str>)
        .map_err(|e| format!("No se pudo abrir el navegador: {e}"))?;

    let code = esperar_callback(listener).await?;
    let (access, refresh, expira) = intercambiar_code(&code, &verifier, &redirect_uri).await?;
    let email = email_de_jwt(&access);
    let sesion = SesionZen {
        access_token: access,
        refresh_token: refresh,
        expira_epoch: expira,
        email: email.clone(),
    };
    guardar_sesion_en(&ruta_sesion(&app)?, &sesion)?;
    Ok(email)
}

/// Desvincula la cuenta (SOLO admin): borra los tokens del disco.
#[tauri::command]
pub async fn zen_salir(
    app: tauri::AppHandle,
    auth: tauri::State<'_, crate::backventanas::auth::AuthState>,
) -> Result<(), String> {
    auth.require_admin()?;
    let ruta = ruta_sesion(&app)?;
    if ruta.exists() {
        std::fs::remove_file(&ruta).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_challenge_deriva_del_verifier() {
        let (verifier, challenge) = generar_pkce();
        assert_eq!(verifier.len(), 43, "32 bytes en b64url sin pad");
        let esperado = b64url(&Sha256::digest(verifier.as_bytes()));
        assert_eq!(challenge, esperado);
    }

    #[test]
    fn url_autorizacion_lleva_pkce_y_loopback() {
        let url = url_autorizacion(18721, "RETO", "ESTADO");
        assert!(url.starts_with("https://auth.opencode.ai/authorize?"));
        assert!(url.contains("client_id=app"));
        assert!(url.contains("redirect_uri=http://localhost:18721/callback"));
        assert!(url.contains("code_challenge=RETO"));
        assert!(url.contains("code_challenge_method=S256"));
    }

    #[test]
    fn extraer_code_agarra_el_code_e_ignora_ruido() {
        let req = "GET /callback?code=ABC123&state=xyz HTTP/1.1\r\nHost: x\r\n\r\n";
        assert_eq!(extraer_code(req).as_deref(), Some("ABC123"));
        assert!(extraer_code("GET /callback HTTP/1.1\r\n\r\n").is_none());
        assert!(extraer_code("basura").is_none());
    }

    #[test]
    fn extraer_tokens_saca_todo_y_expira_a_futuro() {
        let antes = ahora_epoch();
        let resp = serde_json::json!({
            "access_token": "ACC",
            "refresh_token": "REF",
            "expires_in": 3600,
        });
        let (a, r, e) = extraer_tokens(&resp).unwrap();
        assert_eq!(a, "ACC");
        assert_eq!(r.as_deref(), Some("REF"));
        assert!(e > antes + 3500 && e <= antes + 3600);
    }

    #[test]
    fn extraer_tokens_sin_expires_in_asume_una_hora() {
        let antes = ahora_epoch();
        let resp = serde_json::json!({ "access_token": "ACC" });
        let (a, r, e) = extraer_tokens(&resp).unwrap();
        assert_eq!(a, "ACC");
        assert!(r.is_none());
        assert!(e >= antes + 3590);
    }

    #[test]
    fn extraer_tokens_sin_access_es_error() {
        let resp = serde_json::json!({ "error": "invalid_grant" });
        assert!(extraer_tokens(&resp).is_err());
    }

    #[test]
    fn sesion_vigente_respeta_margen_de_5_min() {
        let ahora = 1_000_000;
        let ok = SesionZen {
            access_token: "x".into(),
            refresh_token: None,
            expira_epoch: ahora + 301,
            email: String::new(),
        };
        assert!(sesion_vigente(&ok, ahora));
        let casi = SesionZen { expira_epoch: ahora + 299, ..ok.clone() };
        assert!(!sesion_vigente(&casi, ahora), "a <5 min ya toca refrescar");
        let vacia = SesionZen { access_token: "  ".into(), expira_epoch: ahora + 9999, ..ok };
        assert!(!sesion_vigente(&vacia, ahora));
    }

    #[test]
    fn guardar_y_cargar_sesion_roundtrip() {
        let ruta = std::env::temp_dir().join(format!("zen_sesion_test_{}.json", std::process::id()));
        let sesion = SesionZen {
            access_token: "ACC".into(),
            refresh_token: Some("REF".into()),
            expira_epoch: 123,
            email: "a@b.c".into(),
        };
        guardar_sesion_en(&ruta, &sesion).unwrap();
        assert_eq!(cargar_sesion_de(&ruta).unwrap(), sesion);
        std::fs::remove_file(&ruta).ok();
    }

    #[test]
    fn email_de_jwt_lee_claim_o_vacio() {
        use base64::engine::Engine;
        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(r#"{"sub":"1","email":"a@b.c"}"#);
        assert_eq!(email_de_jwt(&format!("h.{payload}.s")), "a@b.c");
        assert_eq!(email_de_jwt("no-es-jwt"), "");
        assert_eq!(email_de_jwt("h.e30.s"), "", "payload sin email");
    }
}

