// ═══════════════════════════════════════════════════════════════════════════
// SECCIÓN DESTINO — A dónde sale el ticket y el pulso del cajón.
// Tarea única: elegir destino (spooler local o térmica en red), ancho de
// papel y abrir el cajón manualmente. Es controlada: el estado vive en
// VentanaCobro porque el auto-print lo consume al cobrar.
// ═══════════════════════════════════════════════════════════════════════════

import { reportarError } from "../../../../services/tauri";
import { notificarExito } from "../../../../components/notificaciones";
import {
  probarRed,
  abrirCajon,
  ANCHOS_PAPEL,
  esImpresoraVirtual,
  type DestinoPrint,
  type ImpresoraInfo,
} from "../../../../services/impresora";

interface SeccionDestinoProps {
  tabPrint: "local" | "red";
  onTabPrint: (t: "local" | "red") => void;
  impresoras: ImpresoraInfo[];
  impresoraSel: string;
  onImpresoraSel: (nombre: string) => void;
  anchoMm: 80 | 58;
  onAnchoMm: (mm: 80 | 58) => void;
  ipRed: string;
  onIpRed: (ip: string) => void;
  puertoRed: number;
  onPuertoRed: (puerto: number) => void;
  ocupado: boolean;
}

export const armarDestinoPrint = (
  tabPrint: "local" | "red",
  impresoraSel: string,
  ipRed: string,
  puertoRed: number,
): DestinoPrint | null => {
  if (tabPrint === "local") {
    if (!impresoraSel) {
      reportarError("Elige una impresora instalada", "Sin selección");
      return null;
    }
    return { Spooler: { nombre: impresoraSel } };
  }
  if (!ipRed.trim()) {
    reportarError("Escribe la IP de la térmica", "Sin IP");
    return null;
  }
  return { Red: { ip: ipRed.trim(), puerto: puertoRed || 9100 } };
};

const SeccionDestino = ({
  tabPrint, onTabPrint,
  impresoras, impresoraSel, onImpresoraSel,
  anchoMm, onAnchoMm,
  ipRed, onIpRed, puertoRed, onPuertoRed,
  ocupado,
}: SeccionDestinoProps) => {
  const handleProbarRed = async () => {
    if (!ipRed.trim()) {
      reportarError("Escribe la IP de la térmica", "Sin IP");
      return;
    }
    try {
      const msg = await probarRed(ipRed.trim(), puertoRed || 9100);
      notificarExito(msg);
    } catch (error) {
      reportarError("Sin conexión con la térmica", error);
    }
  };

  const handleAbrirCajon = async () => {
    const destino = armarDestinoPrint(tabPrint, impresoraSel, ipRed, puertoRed);
    if (!destino) return;
    try {
      const msg = await abrirCajon(destino);
      notificarExito(msg);
    } catch (error) {
      reportarError("No se pudo abrir el cajón", error);
    }
  };

  return (
    <div className="bg-neutral-900 rounded-3xl p-5 space-y-3">
      <div className="flex bg-white/10 rounded-xl p-1">
        <button
          onClick={() => onTabPrint("local")}
          className={`flex-1 py-2 text-[9px] font-black uppercase tracking-widest rounded-lg transition-all ${
            tabPrint === "local" ? "bg-white text-neutral-900" : "text-neutral-400"
          }`}
        >
          Local
        </button>
        <button
          onClick={() => onTabPrint("red")}
          className={`flex-1 py-2 text-[9px] font-black uppercase tracking-widest rounded-lg transition-all ${
            tabPrint === "red" ? "bg-white text-neutral-900" : "text-neutral-400"
          }`}
        >
          Red
        </button>
      </div>

      {tabPrint === "local" ? (
        impresoras.length === 0 ? (
          <p className="text-[9px] font-bold text-neutral-400 text-center uppercase tracking-widest py-2">
            Sin impresoras instaladas en Windows
          </p>
        ) : (
          <>
            <select
              value={impresoraSel}
              onChange={(e) => onImpresoraSel(e.target.value)}
              className="w-full px-3 py-2.5 bg-white/10 border border-white/10 rounded-xl text-[11px] font-bold text-white focus:outline-none"
            >
              {impresoras.map((imp) => (
                <option key={imp.nombre} value={imp.nombre} className="text-neutral-900">
                  {imp.nombre}{imp.predeterminada ? " (default)" : ""}
                </option>
              ))}
            </select>
            {impresoraSel && esImpresoraVirtual(impresoraSel) && (
              <p className="text-[9px] font-black text-amber-400 text-center uppercase tracking-widest">
                Ojo: es virtual (PDF/XPS), no imprime tickets — elige tu térmica
              </p>
            )}
          </>
        )
      ) : (
        <div className="flex gap-2">
          <input
            value={ipRed}
            onChange={(e) => onIpRed(e.target.value)}
            placeholder="192.168.1.50"
            className="flex-1 min-w-0 px-3 py-2.5 bg-white/10 border border-white/10 rounded-xl text-[11px] font-bold text-white placeholder:text-neutral-500 focus:outline-none"
          />
          <input
            type="number"
            value={puertoRed}
            onChange={(e) => onPuertoRed(Number(e.target.value))}
            className="w-20 px-3 py-2.5 bg-white/10 border border-white/10 rounded-xl text-[11px] font-bold text-white text-center focus:outline-none"
          />
          <button
            onClick={handleProbarRed}
            className="px-3 py-2.5 text-[9px] font-black uppercase tracking-widest rounded-xl bg-white/10 text-white hover:bg-white/20 transition-colors"
          >
            Probar
          </button>
        </div>
      )}

      <div className="flex gap-2">
        <div className="flex bg-white/10 rounded-xl p-1 shrink-0">
          {ANCHOS_PAPEL.map((a) => (
            <button
              key={a.mm}
              onClick={() => onAnchoMm(a.mm)}
              className={`px-2.5 py-2 text-[9px] font-black rounded-lg transition-all ${
                anchoMm === a.mm ? "bg-white text-neutral-900" : "text-neutral-400"
              }`}
            >
              {a.label}
            </button>
          ))}
        </div>
        <button
          onClick={handleAbrirCajon}
          disabled={ocupado}
          title="Manda el pulso de apertura al cajón (va conectado a la térmica)"
          className="flex-1 py-3 rounded-xl bg-white/10 text-white text-[9px] font-black uppercase tracking-[0.2em] hover:bg-white/20 transition-all disabled:opacity-40"
        >
          🗄 Abrir cajón
        </button>
      </div>
    </div>
  );
};

export default SeccionDestino;
