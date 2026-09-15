// ═══════════════════════════════════════════════════════════════════════════
// ATAJOS · MODAL F8 — Rejilla de cuadritos gorditos con los atajos.
// Cuadrito: chip de tecla + etiqueta; hover invierte a negro/blanco;
// click ejecuta y cierra. Los no listos (F6) se ven apagados. Todo sale
// de TABLA_ATAJOS. Escape cierra.
// ═══════════════════════════════════════════════════════════════════════════

import { useEffect } from "react";
import { MorphIcon } from "morphicons/react";
import { TABLA_ATAJOS, type AccionAtajoMenu } from "./tabla-atajos";
import { ICONO_AYUDA, ICONO_EQUIS } from "../../components/ui";

interface ModalAtajosProps {
  onClose: () => void;
  onAccion: (a: AccionAtajoMenu) => void;
}

export default function ModalAtajos({ onClose, onAccion }: ModalAtajosProps) {
  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [onClose]);

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4" onClick={onClose}>
      <div
        className="bg-white rounded-[2.5rem] shadow-2xl w-full max-w-[700px] overflow-hidden animate-in zoom-in-95 fade-in duration-200 max-h-[90vh] overflow-y-auto custom-scrollbar"
        onClick={(e) => e.stopPropagation()}
      >
        {/* HEADER OSCURO */}
        <div className="bg-neutral-950 px-8 pt-7 pb-6 text-center relative overflow-hidden">
          <div className="absolute -top-10 -right-10 w-40 h-40 bg-white/[0.04] rounded-full blur-2xl" />
          <button
            onClick={onClose}
            className="absolute top-5 right-5 p-2 rounded-xl hover:bg-white/10 text-neutral-500 hover:text-white transition-all"
            title="Cerrar (Esc)"
          >
            <MorphIcon icon={ICONO_EQUIS} size={16} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
          </button>
          <div className="w-12 h-12 mx-auto bg-white/10 rounded-2xl flex items-center justify-center mb-3">
            <MorphIcon icon={ICONO_AYUDA} size={20} strokeWidth={2.2} spring="smooth" className="text-white" />
          </div>
          <h2 className="text-lg font-black text-white uppercase tracking-tight">Atajos de caja</h2>
          <p className="text-[9px] font-black text-neutral-500 uppercase tracking-[0.25em] mt-1">
            Todo sin soltar el teclado
          </p>
        </div>

        <div className="p-7">
          <div className="grid grid-cols-3 gap-3">
            {TABLA_ATAJOS.filter((a) => a.enMenu).map((a) => (
              <button
                key={a.tecla}
                disabled={!a.listo}
                title={a.descripcion}
                onClick={() => {
                  if (!a.listo || !a.accion) return;
                  onAccion(a.accion);
                }}
                className={`group aspect-square rounded-3xl border-2 flex flex-col items-center justify-center gap-2 transition-all duration-200 active:scale-95 ${
                  a.listo
                    ? "bg-white border-neutral-200 hover:bg-neutral-950 hover:border-neutral-950 hover:shadow-xl cursor-pointer"
                    : "bg-neutral-50 border-neutral-100 opacity-40 cursor-not-allowed"
                }`}
              >
                <span className={`px-2 py-1 text-[10px] font-black rounded-lg transition-colors ${
                  a.listo ? "bg-neutral-950 text-white group-hover:bg-white group-hover:text-neutral-950" : "bg-neutral-200 text-neutral-400"
                }`}>
                  {a.tecla}
                </span>
                <span className={`text-[10px] font-black uppercase tracking-widest transition-colors ${
                  a.listo ? "text-neutral-900 group-hover:text-white" : "text-neutral-300"
                }`}>
                  {a.etiqueta}
                </span>
              </button>
            ))}
          </div>
          <p className="text-[8px] text-neutral-400 font-bold uppercase tracking-widest text-center mt-4">
            Pica un cuadrito para ejecutar · Esc para cerrar
          </p>
        </div>
      </div>
    </div>
  );
}
