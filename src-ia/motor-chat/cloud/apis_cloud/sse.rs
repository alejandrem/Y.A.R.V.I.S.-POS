// ============================================================
// sse — Lectura de un cuerpo SSE: emite cada línea `data: ...`
// (sin el prefijo) tal como llega del proveedor. Parte de apis_cloud.
// ============================================================

use futures_util::StreamExt;

use super::errores::ErrorCloud;

/// Lee un cuerpo SSE y emite cada línea `data: ...` (sin el prefijo).
pub(crate) fn sse_lineas(
    resp: reqwest::Response,
) -> impl futures_util::Stream<Item = Result<String, ErrorCloud>> {
    async_stream::stream! {
        let mut stream = resp.bytes_stream();
        let mut buf: Vec<u8> = Vec::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| ErrorCloud::Red(e.to_string()))?;
            buf.extend_from_slice(&chunk);
            while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                let mut line: Vec<u8> = buf.drain(..=pos).collect();
                line.pop();
                let line = String::from_utf8_lossy(&line).trim().to_string();
                if line.is_empty() {
                    continue;
                }
                yield Ok(line);
            }
        }
        if !buf.is_empty() {
            let line = String::from_utf8_lossy(&buf).trim().to_string();
            if !line.is_empty() {
                yield Ok(line);
            }
        }
    }
}
