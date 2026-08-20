// ─────────────────────────────────────────────────────────────────────────────
// ICONOS MORPH SHARED (morphicons/react)
// ─────────────────────────────────────────────────────────────────────────────
// Única fuente de verdad de los iconos morpheables de toda la app.
// Cualquier módulo los importa de aquí (NO definir iconos por carpeta).
// Los iconos son paths de lucide-style y sirven como `IconInput` de
// <MorphIcon>. Nombres de variables en español.
//
// Uso:
//   import { ICONO_CHECK, ICONO_BUSCAR } from "../../icons";
//   <MorphIcon icon={ICONO_CHECK} size={16} strokeWidth={2} spring="smooth" />
// ─────────────────────────────────────────────────────────────────────────────
import type { IconInput } from "morphicons/react";

/* ───────────── MORPH BÁSICOS / SISTEMA ───────────── */

// Palomita de éxito / completado
export const ICONO_CHECK: IconInput = "M20 6 9 17l-5-5";

// Palomita dentro de un círculo (confirmado)
export const ICONO_CHECK_CIRCULO: IconInput = "M22 11.08V12a10 10 0 1 1-5.93-9.14 M22 4 12 14.01l-3-3";

// Signo de suma / añadir
export const ICONO_MAS: IconInput = "M12 5v14M5 12h14";

// Signo de suma en círculo
export const ICONO_MAS_CIRCULO: IconInput = "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z M12 8v8 M8 12h8";

// Signo de resta / quitar
export const ICONO_RESTA: IconInput = "M5 12h14";

// Signo de resta en círculo
export const ICONO_RESTA_CIRCULO: IconInput = "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z M8 12h8";

// Equis / cerrar / cancelar
export const ICONO_EQUIS: IconInput = "M18 6 6 18M6 6l12 12";
export const ICONO_CERRAR: IconInput = "M18 6 6 18M6 6l12 12";

// Flecha a la derecha (siguiente / entrar)
export const ICONO_FLECHA: IconInput = "M5 12h14M12 5l7 7-7 7";

// Flecha de regreso (historial / volver)
export const ICONO_HISTORIAL: IconInput =
  "M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8M3 3v5h5M12 7v5l4 2";

// Reloj / hora / turno
export const ICONO_RELOJ: IconInput = "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z M12 6v6l4 2";

// Calendario / fechas
export const ICONO_CALENDARIO: IconInput =
  "M8 2v4 M16 2v4 M3 10h18 M5 4h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2z";

// Campana / notificaciones / avisos
export const ICONO_CAMPANA: IconInput =
  "M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9 M13.73 21a2 2 0 0 1-3.46 0";

// Alarma / triángulo de peligro
export const ICONO_ALERTA: IconInput =
  "M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z M12 9v4 M12 17h.01";

// Alarma en círculo / error
export const ICONO_ALERTA_CIRCULO: IconInput = "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z M12 8v4 M12 16h.01";

// Información / detalle
export const ICONO_INFO: IconInput = "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z M12 16v-4 M12 8h.01";

// Signo de interrogación / ayuda
export const ICONO_AYUDA: IconInput = "M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3 M12 17h.01";

// Menú / lista / opciones
export const ICONO_MENU: IconInput = "M4 6h16 M4 12h16 M4 18h16";

// Lupa / buscar
export const ICONO_BUSCAR: IconInput = "M21 21l-4.35-4.35 M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16z";

// Filtro / seleccionar
export const ICONO_FILTRO: IconInput = "M22 3H2l8 9.46V19l4 2v-8.54z";

// Etiqueta / categoría / precio
export const ICONO_ETIQUETA: IconInput =
  "M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.83z M7 7h.01";

// Código / desarrollador
export const ICONO_CODIGO: IconInput = "M16 18l6-6-6-6 M8 6l-6 6 6 6";

// Link / enlace / vincular
export const ICONO_ENLACE: IconInput =
  "M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71 M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71";

/* ───────────── MORPH DE NAVEGACIÓN / UBICACIÓN ───────────── */

// Inicio / casa
export const ICONO_INICIO: IconInput = "M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z M9 22V12h6v10";

// Ubicación / pin del mapa
export const ICONO_UBICACION: IconInput =
  "M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z M12 13a3 3 0 1 0 0-6 3 3 0 0 0 0 6z";

// Mundo / global / país
export const ICONO_MUNDO: IconInput =
  "M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20z M2 12h20 M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z";

// Nube / cloud / sincronizar
export const ICONO_NUBE: IconInput = "M18 10h-1.26A8 8 0 1 0 9 20h9a5 5 0 0 0 0-10z";

/* ───────────── MORPH DE CONTACTO / COMUNICACIÓN ───────────── */

// Correo / email
export const ICONO_CORREO: IconInput =
  "M4 4h16a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2z M22 6l-10 7L2 6";

// Teléfono
export const ICONO_TELEFONO: IconInput =
  "M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72c.127.96.361 1.903.7 2.81a2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45c.907.339 1.85.573 2.81.7A2 2 0 0 1 22 16.92z";

/* ───────────── MORPH DE DOCUMENTOS / ARCHIVOS ───────────── */

// Documento / archivo
export const ICONO_DOCUMENTO: IconInput = "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z M14 2v6h6";

// Documento nuevo / crear archivo
export const ICONO_DOCUMENTO_NUEVO: IconInput =
  "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z M14 2v6h6 M12 18v-6 M9 15h6";

// Carpeta / agrupar
export const ICONO_CARPETA: IconInput =
  "M22 20a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z";

// Libro / bitácora / documentación
export const ICONO_LIBRO: IconInput = "M3 3h5v18H3z M11 5h10 M11 9h10 M11 13h10 M11 17h10";

// Fotografía / imagen
export const ICONO_FOTO: IconInput =
  "M5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z M8.5 10.5a1.5 1.5 0 1 1 0-3 1.5 1.5 0 0 1 0 3z M21 15l-5-5L5 21";

// Cámara / foto
export const ICONO_CAMARA: IconInput =
  "M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z M12 17a4 4 0 1 1 0-8 4 4 0 0 1 0 8z";

// Impresora
export const ICONO_IMPRESORA: IconInput =
  "M6 9V2h12v7 M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2 M6 14h12v8H6z";

// Descargar
export const ICONO_DESCARGAR: IconInput = "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4 M7 10l5 5 5-5 M12 15V3";

// Subir
export const ICONO_SUBIR: IconInput = "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4 M17 8l-5-5-5 5 M12 3v12";

/* ───────────── MORPH DE VENTAS / FINANZAS ───────────── */

// Dólar / dinero
export const ICONO_DOLAR: IconInput = "M12 1v22 M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6";

// Tendencia / gráfica de crecimiento
export const ICONO_TRENDING: IconInput = "M23 6l-9.5 9.5-5-5L1 18 M17 6h6v6";

// Gráfica de barras / estadísticas
export const ICONO_GRAFICA: IconInput = "M12 20V10 M18 20V4 M6 20v-4";

// Premio / bono / reconocimiento
export const ICONO_PREMIO: IconInput =
  "M8 21h8 M12 17v4 M7 4h10v4a5 5 0 0 1-10 0V4z M7 5H4a2 2 0 0 0 2 4h3 M17 5h3a2 2 0 0 1-2 4h-3";

// Trofeo / logro
export const ICONO_TROFEO: IconInput =
  "M6 9H4.5a5.5 5.5 0 0 1 0-11H6v11z M18 9h1.5a5.5 5.5 0 0 0 0-11H18v11z M12 20V8 M6 17h12 M8 21h8";

// Estrella / favorito / valoración
export const ICONO_ESTRELLA: IconInput =
  "M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01z";

// Corazón / favorito
export const ICONO_CORAZON: IconInput =
  "M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z";

// Billete / efectivo
export const ICONO_BILLETE: IconInput =
  "M2 6h20v12H2z M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z M6 9h.01 M18 15h.01";

// Tarjeta de crédito
export const ICONO_TARJETA: IconInput = "M1 4h22v16H1z M1 10h22";

// Porcentaje / descuento
export const ICONO_PORCENTAJE: IconInput =
  "M19 5 5 19 M6.5 9a2.5 2.5 0 1 1 0-5 2.5 2.5 0 0 1 0 5z M17.5 20a2.5 2.5 0 1 1 0-5 2.5 2.5 0 0 1 0 5z";

// Bolsa de compras
export const ICONO_BOLSA: IconInput =
  "M6 2 3 6v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V6l-3-4z M3 6h18 M16 10a4 4 0 0 1-8 0";

// Carrito de compras
export const ICONO_CARRITO: IconInput =
  "M9 22a1 1 0 1 0 0-2 1 1 0 0 0 0 2z M20 22a1 1 0 1 0 0-2 1 1 0 0 0 0 2z M1 1h4l2.68 13.39a2 2 0 0 0 2 1.61h9.72a2 2 0 0 0 2-1.61L23 6H6";

// Calculadora
export const ICONO_CALCULADORA: IconInput =
  "M4 2h16a2 2 0 0 1 2 2v16a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2z M8 6h8 M7 12h.01 M12 12h.01 M17 12h.01 M7 16h.01 M12 16h.01 M17 16h.01";

/* ───────────── MORPH DE LOGÍSTICA / INVENTARIO ───────────── */

// Caja / paquete
export const ICONO_CAJA: IconInput =
  "M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z M3.27 6.96 12 12.01l8.73-5.05 M12 22.08V12";

// Camión / envío / reparto
export const ICONO_CAMION: IconInput =
  "M1 3h15v13H1z M16 8h4l3 3v5h-7z M5.5 21a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5z M18.5 21a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5z";

// Base de datos / respaldos
export const ICONO_BASE_DATOS: IconInput =
  "M12 8c4.97 0 9-1.343 9-3s-4.03-3-9-3-9 1.343-9 3 4.03 3 9 3z M21 12c0 1.66-4 3-9 3s-9-1.34-9-3 M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5";

// Escáner / capturar
export const ICONO_ESCANER: IconInput =
  "M3 7V5a2 2 0 0 1 2-2h2 M17 3h2a2 2 0 0 1 2 2v2 M21 17v2a2 2 0 0 1-2 2h-2 M7 21H5a2 2 0 0 1-2-2v-2 M3 12h18";

// Código de barras
export const ICONO_CODIGO_BARRAS: IconInput = "M7 3v18 M12 3v18 M17 3v18 M4 3v18 M21 3v18";

/* ───────────── MORPH DE HARDWARE / TECNOLOGÍA ───────────── */

// Celular / smartphone
export const ICONO_CELULAR: IconInput =
  "M12 18h.01 M16 2H8a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2V4a2 2 0 0 0-2-2z";

// Pantalla / monitor
export const ICONO_PANTALLA: IconInput = "M2 3h20v14H2z M8 21h8 M12 17v4";

/* ───────────── MORPH DE ENERGÍA / NATURALEZA ───────────── */

// Sol / día
export const ICONO_SOL: IconInput =
  "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8z M12 2v2 M12 20v2 M4.93 4.93l1.41 1.41 M17.66 17.66l1.41 1.41 M2 12h2 M20 12h2 M4.93 19.07l1.41-1.41 M17.66 6.34l1.41-1.41";

// Luna / noche
export const ICONO_LUNA: IconInput = "M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z";

// Fuego / popular / caliente
export const ICONO_FUEGO: IconInput =
  "M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.38-.5-2-1-3-1.072-2.143-.224-4.054 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.153.433-2.294 1-3a2.5 2.5 0 0 0 2.5 2.5z";

/* ───────────── MORPH DE PERSONAS / SOCIAL ───────────── */

// Grupo de personas / usuarios
export const ICONO_USUARIOS: IconInput =
  "M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2 M9 7a4 4 0 1 0 0 8 4 4 0 0 0 0-8 M23 21v-2a4 4 0 0 0-3-3.87 M16 3.13a4 4 0 0 1 0 7.75";

// Una persona / usuario único
export const ICONO_USUARIO: IconInput =
  "M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2 M12 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8z";

// Ojo / ver
export const ICONO_OJO: IconInput = "M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z";

// Ojo oculto / no ver
export const ICONO_OJO_OCULTO: IconInput =
  "M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19M14.12 14.12a3 3 0 1 1-4.24-4.24M1 1l22 22";

// Mano arriba / aprobado / me gusta
export const ICONO_APROBADO: IconInput =
  "M7 10v12 M15 5.88 14 10h5.83a2 2 0 0 1 1.92 2.56l-2.33 8A2 2 0 0 1 17.5 22H4a2 2 0 0 1-2-2v-8a2 2 0 0 1 2-2h2.76a2 2 0 0 0 1.79-1.11L12 2a3.13 3.13 0 0 1 3 3.88z";

// Objetivo / meta / precisión
export const ICONO_TARGET: IconInput =
  "M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20z M12 6a6 6 0 1 0 0 12 6 6 0 0 0 0-12z M12 10a2 2 0 1 0 0 4 2 2 0 0 0 0-4z";

// Candado / seguro / contraseña
export const ICONO_CANDADO: IconInput =
  "M18 8h-1V6a5 5 0 0 0-10 0v2H6a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V10a2 2 0 0 0-2-2zm-6 4v4";

/* ───────────── MORPH DE TAREAS / EDICIÓN ───────────── */

// Editar / lápiz
export const ICONO_EDITAR: IconInput = "M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z";

// Borrar / papelera
export const ICONO_BORRAR: IconInput =
  "M3 6h18 M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6 M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2";

// Reintentar / recargar
export const ICONO_REINICIAR: IconInput =
  "M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8M3 3v5h5";

/* ───────────── MORPH DE LA TIENDA / ESTABLECIMIENTO ───────────── */

// Tienda / negocio
export const ICONO_TIENDA: IconInput =
  "M2 7l4.41-4.41A2 2 0 0 1 7.83 2h8.34a2 2 0 0 1 1.42.59L22 7M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8M2 7h20v3a2 2 0 0 1-2 2 2 2 0 0 1-2-2 2 2 0 0 1-2 2 2 2 0 0 1-2-2 2 2 0 0 1-2 2 2 2 0 0 1-2-2 2 2 0 0 1-2 2 2 2 0 0 1-2-2 2 2 0 0 1-2 2v0";

// Regalo / promoción
export const ICONO_REGALO: IconInput =
  "M20 12v10H4V12 M2 7h20v5H2z M12 22V7 M12 7H7.5a2.5 2.5 0 0 1 0-5C11 2 12 7 12 7z M12 7h4.5a2.5 2.5 0 0 0 0-5C13 2 12 7 12 7z";

/* ───────────── MORPH DE IA / ENVÍO / COMUNICACIÓN ───────────── */

// Reactor / enviar
export const ICONO_ENVIAR: IconInput = "M22 2 11 13M22 2 15 22 11 13 2 9 22 2Z";

// Pausa / detener momentáneamente
export const ICONO_PAUSA: IconInput = "M9 9v6M15 9v6";

// Robot / IA
export const ICONO_ROBOT: IconInput = [
  ["rect", { width: "16", height: "12", x: "4", y: "8", rx: "2" }],
  ["path", { d: "M12 8V4H8" }],
  ["path", { d: "M2 14h2" }],
  ["path", { d: "M20 14h2" }],
  ["path", { d: "M15 13v2" }],
  ["path", { d: "M9 13v2" }],
];

// Engranaje / configuración
export const ICONO_ENGRANAJE: IconInput = [
  ["path", { d: "M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" }],
  ["circle", { cx: "12", cy: "12", r: "3" }],
];