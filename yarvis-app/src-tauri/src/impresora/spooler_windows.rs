// ============================================================
// impresora/spooler_windows.rs — Spooler RAW solo Windows.
//
// Lista con EnumPrintersW (locales + conexiones) y escribe con
// OpenPrinterW + StartDocPrinterW(datatype RAW) + WritePrinter.
// Referencia de patron: a-eid/tauri-pos-printer (Windows escribe
// directo al share, sin dialogos GDI).
//
// Todo lo bloqueante corre en spawn_blocking desde commands.rs;
// aqui solo hay llamadas WinAPI sincronas que devuelven Result.
// ============================================================

use windows::core::{HSTRING, PCWSTR, PWSTR};
use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, HANDLE};
use windows::Win32::Graphics::Printing::{
    AbortPrinter, ClosePrinter, EndDocPrinter, EndPagePrinter, EnumPrintersW, GetDefaultPrinterW,
    OpenPrinterW, StartDocPrinterW, StartPagePrinter, WritePrinter, DOC_INFO_1W,
    PRINTER_ACCESS_USE, PRINTER_DEFAULTSW, PRINTER_ENUM_CONNECTIONS, PRINTER_ENUM_LOCAL,
    PRINTER_INFO_2W,
};

/// Una impresora vista por el spooler.
#[derive(Debug, Clone)]
pub struct ImpresoraSistema {
    pub nombre: String,
    pub predeterminada: bool,
}

fn ancho(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn pwstr_de(vec: &mut [u16]) -> PWSTR {
    PWSTR(vec.as_mut_ptr())
}

/// Mensaje legible con codigo Win32 (el numero solo no le dice nada
/// al tendero). OJO: capturar INMEDIATAMENTE despues del fallo: cualquier
/// llamada Win32 intermedia (ClosePrinter, EndDoc...) pisa GetLastError
/// y el mensaje saldria con "Win32 error 0".
fn win32_mensaje(contexto: &str) -> String {
    let err = windows::core::Error::from_win32();
    format!("{} (Win32 error {}: {})", contexto, err.code().0, err.message())
}

fn nombre_default() -> String {
    unsafe {
        let mut len: u32 = 0;
        // Primera llamada para saber el tamaño (falla a proposito).
        let _ = GetDefaultPrinterW(PWSTR::null(), &mut len);
        if len == 0 {
            return String::new();
        }
        let mut buf: Vec<u16> = vec![0; len as usize];
        if GetDefaultPrinterW(pwstr_de(&mut buf), &mut len).as_bool() {
            let fin = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
            String::from_utf16_lossy(&buf[..fin])
        } else {
            String::new()
        }
    }
}

/// Enumera impresoras instaladas (locales + red). No abre dialogs.
pub fn listar_impresoras_sistema() -> Result<Vec<ImpresoraSistema>, String> {
    unsafe {
        let flags = PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS;
        let mut necesitados: u32 = 0;
        let mut devueltas: u32 = 0;
        // Sondeo de tamaño: se espera ERROR_INSUFFICIENT_BUFFER. Cualquier
        // otro codigo (spooler parado, permisos) es error REAL y antes se
        // tragaba devolviendo "sin impresoras".
        match EnumPrintersW(
            flags,
            PCWSTR::null(),
            2,
            None,
            &mut necesitados,
            &mut devueltas,
        ) {
            Ok(()) => {}
            // Tamaño sondeado: el unico "error" esperado en esta llamada.
            Err(e) if e.code() == ERROR_INSUFFICIENT_BUFFER.into() => {}
            Err(e) => {
                return Err(format!(
                    "No se pudo enumerar impresoras. Revisa que el spooler este activo (Win32 error {}: {})",
                    e.code().0,
                    e.message()
                ));
            }
        }
        if necesitados == 0 {
            return Ok(vec![]);
        }
        // Vec<u64> a proposito: alinear a 8 como PRINTER_INFO_2W (un
        // Vec<u8> casteado seria UB por alineacion).
        let celdas = (necesitados as usize).div_ceil(8);
        let mut buf: Vec<u64> = vec![0; celdas];
        let bytes = std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, celdas * 8);
        EnumPrintersW(
            flags,
            PCWSTR::null(),
            2,
            Some(bytes),
            &mut necesitados,
            &mut devueltas,
        )
        .map_err(|e| format!("No se pudo enumerar impresoras: {e}"))?;
        let infos = std::slice::from_raw_parts(
            buf.as_ptr() as *const PRINTER_INFO_2W,
            devueltas as usize,
        );
        let default = nombre_default();
        let mut lista = Vec::with_capacity(infos.len());
        for info in infos {
            let nombre = info
                .pPrinterName
                .to_string()
                .unwrap_or_default()
                .trim()
                .to_string();
            if nombre.is_empty() {
                continue;
            }
            lista.push(ImpresoraSistema {
                predeterminada: !default.is_empty() && nombre == default,
                nombre,
            });
        }
        // La default primero para que el selector la preseleccione.
        lista.sort_by(|a, b| b.predeterminada.cmp(&a.predeterminada));
        Ok(lista)
    }
}

/// Manda bytes tal cual al spooler con datatype RAW.
/// `nombre` es el nombre del share local ("Mi Termica 80mm"), sin `\\`.
pub fn enviar_bytes_raw(nombre: &str, bytes: &[u8]) -> Result<(), String> {
    let nombre = nombre.trim();
    if nombre.is_empty() {
        return Err("Elige una impresora instalada.".into());
    }
    if bytes.is_empty() {
        return Err("Nada que imprimir: el ticket viene vacio.".into());
    }
    if bytes.len() > 512 * 1024 {
        return Err("Ticket demasiado grande para Fase 1 (limite 512KB).".into());
    }

    unsafe {
        let nombre_h = HSTRING::from(nombre);
        let mut tipo_w = ancho("RAW");
        let mut doc_w = ancho("YARVIS Lista");
        let defaults = PRINTER_DEFAULTSW {
            pDatatype: pwstr_de(&mut tipo_w),
            pDevMode: std::ptr::null_mut(),
            DesiredAccess: PRINTER_ACCESS_USE,
        };
        let mut handle = HANDLE::default();
        OpenPrinterW(&nombre_h, &mut handle, Some(&defaults)).map_err(|e| {
            format!("No se pudo abrir '{nombre}'. Revisa que este instalada y encendida: {e}")
        })?;

        let doc = DOC_INFO_1W {
            pDocName: pwstr_de(&mut doc_w),
            pOutputFile: PWSTR::null(),
            pDatatype: pwstr_de(&mut tipo_w),
        };
        // StartDoc devuelve job id (0 = fallo). El mensaje se captura
        // ANTES de cerrar: ClosePrinter pisaria GetLastError.
        let job = StartDocPrinterW(handle, 1, &doc);
        if job == 0 {
            let msg = win32_mensaje("El spooler rechazo el trabajo");
            let _ = ClosePrinter(handle);
            return Err(msg);
        }
        if !StartPagePrinter(handle).as_bool() {
            let msg = win32_mensaje("No se pudo iniciar pagina");
            let _ = EndDocPrinter(handle);
            let _ = ClosePrinter(handle);
            return Err(msg);
        }
        let mut escritos: u32 = 0;
        let ok = WritePrinter(
            handle,
            bytes.as_ptr() as *const std::ffi::c_void,
            bytes.len() as u32,
            &mut escritos,
        )
        .as_bool();
        // Verificar ANTES de confirmar: confirmar un trabajo trunco saca
        // papel a medias. AbortPrinter lo cancela limpio.
        if !ok || escritos as usize != bytes.len() {
            let msg = if ok {
                format!(
                    "Solo se escribieron {}/{} bytes al spooler",
                    escritos,
                    bytes.len()
                )
            } else {
                win32_mensaje("Fallo escribiendo al spooler")
            };
            let _ = AbortPrinter(handle);
            let _ = ClosePrinter(handle);
            return Err(format!("{msg}. Se cancelo el trabajo para no sacar papel a medias."));
        }
        let _ = EndPagePrinter(handle);
        let _ = EndDocPrinter(handle);
        let _ = ClosePrinter(handle);
        Ok(())
    }
}
