// Bloque "Asistencia de hoy" del detalle: barra con preExtra/trabajo/
// postExtra y markers de login, entrada oficial y salida.
import { geometriaBarra, fmtHM, type MiTurno } from "../../../../components/turno";
import { MorphIcon } from "morphicons/react";
import { ICONO_RELOJ } from "../../../../components/ui";

interface DetalleAsistenciaProps {
  asistenciaDetalle: MiTurno | null;
  ahora: Date;
}

export const DetalleAsistencia = ({ asistenciaDetalle, ahora }: DetalleAsistenciaProps) => {
  const barraDetalle = geometriaBarra(asistenciaDetalle, ahora);
  return (
    <div className="bg-white rounded-[2.5rem] border border-neutral-200 p-6 sm:p-8 shadow-sm">
      <div className="flex items-center gap-3 mb-6 flex-wrap">
        <div className="w-10 h-10 bg-neutral-950 rounded-2xl flex items-center justify-center shadow-md shrink-0">
          <MorphIcon icon={ICONO_RELOJ} size={16} strokeWidth={2.2} spring="smooth" />
        </div>
        <div>
          <h4 className="text-xs font-black text-neutral-900 uppercase tracking-widest">Asistencia de hoy</h4>
          <p className="text-[9px] text-neutral-400 font-bold uppercase tracking-wider">La misma barra que ve el empleado</p>
        </div>
        {barraDetalle?.enExtra && (
          <span className="ml-auto inline-flex items-center gap-1.5 px-3 py-1.5 bg-emerald-50 border border-emerald-200 rounded-xl text-[10px] font-black uppercase tracking-widest text-emerald-600 animate-pulse">
            Extra: +{Math.floor(barraDetalle.extraMinutos / 60)}h {barraDetalle.extraMinutos % 60}m
          </span>
        )}
        {barraDetalle?.llegoPuntual && (
          <span className="ml-auto inline-flex items-center gap-1.5 px-3 py-1.5 bg-sky-50 border border-sky-200 rounded-xl text-[10px] font-black uppercase tracking-widest text-sky-600">
            Llegó temprano
          </span>
        )}
      </div>

      {!barraDetalle ? (
        <div className="py-8 text-center bg-neutral-50 rounded-2xl border border-dashed border-neutral-200">
          <p className="text-sm font-black uppercase tracking-widest text-neutral-400">Hoy no tiene turno asignado</p>
          <p className="text-[10px] font-bold text-neutral-300 mt-1.5">Día de descanso o sin horario configurado</p>
        </div>
      ) : (
        <>
          <div className="flex items-center gap-4 mb-4">
            <div className="text-center shrink-0">
              <p className="text-2xl font-black text-neutral-900">{fmtHM(barraDetalle.inicio)}</p>
              <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mt-1">Entrada</p>
            </div>

            <div className="flex-1">
              <div className="relative h-4 bg-neutral-100 rounded-full border border-neutral-200 overflow-visible">
                {barraDetalle.preExtraActivo && (
                  <div
                    className="absolute inset-y-0 bg-emerald-400 transition-all duration-700 ease-out"
                    style={{ left: `${barraDetalle.loginPct}%`, width: `${Math.max(0, barraDetalle.preExtraPct)}%`, borderRadius: "999px 0 0 999px" }}
                  />
                )}
                <div
                  className="absolute inset-y-0 bg-neutral-900 rounded-full transition-all duration-700 ease-out"
                  style={{ left: `${barraDetalle.inicioPct}%`, width: `${Math.max(0, barraDetalle.trabajoPct)}%` }}
                />
                {barraDetalle.enExtraPost && (
                  <div
                    className="absolute inset-y-0 bg-emerald-500 transition-all duration-700 ease-out"
                    style={{ left: `${barraDetalle.finPct}%`, width: `${Math.max(0, barraDetalle.postExtraPct)}%`, borderRadius: "0 999px 999px 0" }}
                  />
                )}
                {barraDetalle.loginPct !== null && (
                  <div
                    className="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-3.5 h-3.5 bg-white border-[3px] border-neutral-900 rounded-full shadow-md z-10"
                    style={{ left: `${barraDetalle.loginPct}%` }}
                    title={`Primer login: ${asistenciaDetalle?.primer_login ?? ""}`}
                  />
                )}
                {barraDetalle.preExtraActivo && (
                  <div
                    className="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-3.5 h-3.5 bg-white border-[3px] border-neutral-900 rounded-full shadow-md z-10"
                    style={{ left: `${barraDetalle.inicioPct}%` }}
                    title="Entrada oficial"
                  />
                )}
                {barraDetalle.enExtraPost && (
                  <div
                    className="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-3.5 h-3.5 bg-white border-[3px] border-emerald-500 rounded-full shadow-md z-10"
                    style={{ left: `${barraDetalle.finPct}%` }}
                    title="Fin de horario — extra en curso"
                  />
                )}
              </div>

              <div className="flex justify-between mt-2">
                <span className="text-[8px] font-black text-neutral-300 uppercase">
                  {asistenciaDetalle?.primer_login
                    ? `Primer login ${asistenciaDetalle.primer_login}${
                        barraDetalle.minutosTarde > 0
                          ? ` · ${barraDetalle.minutosTarde} min tarde`
                          : barraDetalle.llegoPuntual
                            ? " · puntual"
                            : ` · ${barraDetalle.minutosTemprano} min temprano (extra)`
                      }`
                    : "Sin registro de entrada hoy"}
                </span>
                <span className={`text-[8px] font-black uppercase ${barraDetalle.enExtra ? "text-emerald-500" : "text-neutral-300"}`}>
                  {barraDetalle.enExtra ? `Progreso: ${Math.round(barraDetalle.trabajoPct)}% + extra` : `Progreso: ${Math.round(barraDetalle.trabajoPct)}%`}
                </span>
              </div>
            </div>

            <div className="text-center shrink-0">
              <p className={`text-2xl font-black ${barraDetalle.enExtra ? "text-emerald-600" : "text-neutral-900"}`}>{fmtHM(barraDetalle.fin)}</p>
              <p className={`text-[9px] font-black uppercase tracking-widest mt-1 ${barraDetalle.enExtra ? "text-emerald-500" : "text-neutral-400"}`}>Salida</p>
            </div>
          </div>
        </>
      )}
    </div>
  );
};
