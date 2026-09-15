// ═══════════════════════════════════════════════════════════════════════════
// REIMPRIMIR · TEXTO — Previsualización 80mm/48cols del ticket a reimprimir.
// Espeja lo que la térmica imprime vía imprimir_ticket_venta (mismo orden:
// encabezado, líneas, total, método, despedida).
// ═══════════════════════════════════════════════════════════════════════════

import type { MiTicketDetalle } from "../../../../services/misventas";

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

/** "YYYY-MM-DD HH:MM:SS" -> "DD/MM/YYYY HH:MM" (igual que cortes). */
function fmtFecha(s: string): string {
  const m = s.match(/(\d{4})-(\d{2})-(\d{2})[ T](\d{2}):(\d{2})/);
  if (!m) return s;
  return `${m[3]}/${m[2]}/${m[1]} ${m[4]}:${m[5]}`;
}

const fmtCant = (c: number) => (Number.isInteger(c) ? String(c) : String(c));

export const folioTicket = (d: MiTicketDetalle): string =>
  d.folio_ticket ?? `TICKET-${String(d.id).padStart(4, "0")}`;

/** Texto completo del ticket para la preview (y referencia de impresión). */
export function textoTicketReimpresion(d: MiTicketDetalle, tienda: string): string {
  const cuerpo = d.items.map((it) =>
    fila(moneda(it.cantidad * it.precio_unitario), `${fmtCant(it.cantidad)}x ${it.producto_nombre}`),
  );
  return [
    "*** TICKET DE VENTA ***",
    tienda,
    folioTicket(d),
    fmtFecha(d.fecha),
    linea(),
    ...cuerpo,
    linea(),
    fila(moneda(d.total), "TOTAL"),
    fila(moneda(d.total), d.metodo_pago),
    "Gracias por su compra",
    linea(),
  ].join("\n");
}
