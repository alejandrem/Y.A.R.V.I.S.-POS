// ═══════════════════════════════════════════════════════════════════════════
// AVISO-RELOJ — Banner del estado del reloj del sistema.
// Solo pinta cuando la hora local retrocede o cambia bruscamente.
// ═══════════════════════════════════════════════════════════════════════════
import type { EstadoHora } from "../../services/turno";

interface AvisoRelojProps {
  hora: EstadoHora | null;
  compacto?: boolean;
}

const AvisoReloj = ({ hora, compacto = false }: AvisoRelojProps) => {
  if (!hora || hora.estado === "ok") return null;

  const esAtrasado = hora.estado === "reloj_atrasado";
  return (
    <div
      role="alert"
      className={`rounded-2xl border px-4 ${compacto ? "py-2.5" : "py-3.5"} mt-4 text-[11px] font-bold leading-relaxed ${
        esAtrasado
          ? "bg-red-50 border-red-200 text-red-700"
          : "bg-amber-50 border-amber-200 text-amber-700"
      }`}
    >
      <p className="font-black uppercase tracking-widest text-[10px] mb-1">
        {esAtrasado ? "Revisa la hora de tu computadora" : "Tu hora puede estar mal"}
      </p>
      <p>{hora.mensaje}</p>
      <p className="mt-1 text-[10px] opacity-80">
        Reloj: {hora.hora_sistema}
        {hora.zona ? ` · Zona: ${hora.zona}` : ""}
        {hora.ultimo_evento ? ` · Último movimiento: ${hora.ultimo_evento}` : ""}
      </p>
      <p className="mt-1 text-[10px] font-black uppercase tracking-wider opacity-70">
        Configuración → Hora e idioma → Fecha y hora
      </p>
    </div>
  );
};

export default AvisoReloj;
