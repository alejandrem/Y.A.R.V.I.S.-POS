// ═══════════════════════════════════════════════════════════════════════════
// EMPLEACORTES · TICKET-TEXTO — Previsualización del corte con el mismo
// formato que imprime el backend (térmica 80mm / 48 columnas). Si cambias
// este layout, cambia también backcortes/impresion.rs para que concuerden.
// ═══════════════════════════════════════════════════════════════════════════

import type { CorteXReporte, CorteZReporte } from "./tipos";

const COLS = 48;

const moneda = (n: number) => `$${Math.max(0, n).toFixed(2)}`;

/** Fila monoespaciada: izquierda recortada + importe a la derecha. */
function fila(importe: string, izquierda: string): string {
  const der = [...importe].length;
  const maxIzq = Math.max(0, COLS - der - 1);
  let izq = [...izquierda].slice(0, maxIzq).join("");
  while ([...izq].length + der < COLS) izq += " ";
  return `${izq}${importe}`;
}

const linea = () => "-".repeat(COLS);

/** "YYYY-MM-DD HH:MM:SS" -> "DD/MM/YYYY HH:MM" (igual que el backend). */
export function fmtFechaCorte(s: string): string {
  const m = s.match(/(\d{4})-(\d{2})-(\d{2})[ T](\d{2}):(\d{2})/);
  if (!m) return s;
  return `${m[3]}/${m[2]}/${m[1]} ${m[4]}:${m[5]}`;
}

const fmtCant = (c: number) => (Number.isInteger(c) ? String(c) : String(c));

function encabezado(tipo: string, tienda: string, cajero: string, numero: number, ancla: string, cierre: string): string[] {
  return [
    `*** CORTE ${tipo} EN MONEDA:MXN ***`,
    tienda,
    `Cajero: ${cajero.toUpperCase()}`,
    linea(),
    `Corte ${tipo} #${numero}`,
    `${fmtFechaCorte(ancla)} - ${fmtFechaCorte(cierre)}`,
    linea(),
    "** VENTAS DEL TURNO **",
  ];
}

function pieTotales(t: CorteXReporte["totales"]): string[] {
  return [
    linea(),
    fila(moneda(t.total_ventas), "Total ventas"),
    fila(moneda(t.total_efectivo), "Efectivo"),
    fila(moneda(t.total_tarjeta), "Tarjeta"),
    fila(moneda(t.total_transferencia), "Transferencia"),
    linea(),
    `Tickets: ${t.num_tickets}`,
    linea(),
  ];
}

/** Texto del corte X: tickets con folio + total. */
export function textoCorteX(r: CorteXReporte, tienda: string, cajero: string): string {
  const cuerpo = r.tickets.map((t) => fila(moneda(t.total), t.folio));
  return [...encabezado("X", tienda, cajero, r.corte_id, r.ancla, r.cierre), ...cuerpo, ...pieTotales(r.totales), "X informativo: no cierra tu turno", linea()].join("\n");
}

/** Texto del corte Z: productos agregados (nombre + cantidad + monto). */
export function textoCorteZ(r: CorteZReporte, tienda: string, cajero: string): string {
  const cuerpo = r.productos.map((p) => fila(moneda(p.monto), `${p.producto_nombre} ${fmtCant(p.cantidad)}x`));
  return [...encabezado("Z", tienda, cajero, r.corte_id, r.ancla, r.cierre), ...cuerpo, ...pieTotales(r.totales), "*** TURNO CERRADO ***", "Contador reiniciado a $0.00", linea()].join("\n");
}
