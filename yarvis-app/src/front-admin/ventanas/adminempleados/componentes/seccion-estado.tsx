// ═══════════════════════════════════════════════════════════════════════════
// SECCIÓN ESTADO — Bloque de estado del empleado (solo modo edición):
// desactivar con confirmación y reactivar. Presentacional: el cambio real
// lo ejecuta el callback onCambiarEstado de ModalEmpleados.
// ═══════════════════════════════════════════════════════════════════════════

import { MorphIcon } from "morphicons/react";
import { ICONO_ALERTA, ICONO_CHECK, ICONO_CERRAR } from "../../../../components/ui";

interface SeccionEstadoProps {
  nombreEmpleado?: string;
  estadoActual: string;
  /** Confirmación de desactivación: vive en ModalEmpleados porque el
   *  guardado exitoso la resetea desde cambiarEstado. */
  confirmando: boolean;
  setConfirmando: (valor: boolean) => void;
  cambiandoEstado: boolean;
  onCambiarEstado: (nuevoEstado: string) => void;
}

const SeccionEstado = ({
  nombreEmpleado,
  estadoActual,
  confirmando,
  setConfirmando,
  cambiandoEstado,
  onCambiarEstado,
}: SeccionEstadoProps) => {

  return (
    <div className={`rounded-2xl p-4 space-y-3 border-2 ${estadoActual === "inactivo" ? "border-red-200 bg-red-50/60" : "border-neutral-200 bg-white"}`}>
      <p className="flex items-center gap-2 text-[10px] font-black text-neutral-500 uppercase tracking-widest">
        <MorphIcon icon={ICONO_ALERTA} size={14} strokeWidth={2.4} spring="smooth" className={estadoActual === "inactivo" ? "text-red-500" : ""} />
        Estado ·{" "}
        <span className={estadoActual === "inactivo" ? "text-red-500" : "text-emerald-600"}>
          {estadoActual === "inactivo" ? "Inactivo" : "Activo"}
        </span>
      </p>

      {!confirmando ? (
        <>
          <button
            type="button"
            onClick={() => setConfirmando(true)}
            disabled={estadoActual === "inactivo"}
            className="w-full inline-flex items-center justify-center gap-2 py-3 rounded-xl bg-red-500 text-white text-[10px] font-black uppercase tracking-[0.15em] hover:bg-red-600 transition-all shadow-lg shadow-red-200 active:scale-[0.98] disabled:opacity-30 disabled:cursor-not-allowed"
          >
            <MorphIcon icon={ICONO_CERRAR} size={14} strokeWidth={2.5} spring="snappy" />
            Desactivar Empleado
          </button>
          {estadoActual === "inactivo" && (
            <button
              type="button"
              onClick={() => onCambiarEstado("activo")}
              disabled={cambiandoEstado}
              className="w-full inline-flex items-center justify-center gap-2 py-3 rounded-xl bg-emerald-500 text-white text-[10px] font-black uppercase tracking-[0.15em] hover:bg-emerald-600 transition-all shadow-lg shadow-emerald-200 active:scale-[0.98] disabled:opacity-40"
            >
              <MorphIcon icon={ICONO_CHECK} size={14} strokeWidth={2.5} spring="snappy" />
              {cambiandoEstado ? "Reactivando..." : "Reactivar Empleado"}
            </button>
          )}
        </>
      ) : (
        /* ADVERTENCIA con Aceptar / Cancelar */
        <div className="space-y-3">
          <div className="bg-white rounded-xl p-4 border border-red-100 space-y-2">
            <p className="text-[11px] font-black text-red-500 uppercase tracking-widest">
              ¿Desactivar a {nombreEmpleado}?
            </p>
            <ul className="text-[10px] font-bold text-neutral-500 space-y-1.5 list-disc list-inside">
              <li><span className="font-black text-neutral-700">No podrá iniciar sesión</span> en el punto de venta.</li>
              <li>Sus ventas, cortes de caja y historial <span className="font-black text-neutral-700">se conservan intactos</span>.</li>
              <li>Dejará de contar para los resúmenes y la nómina activa.</li>
              <li>Puedes reactivarlo en cualquier momento desde aquí mismo.</li>
            </ul>
          </div>
          <div className="grid grid-cols-2 gap-2">
            <button
              type="button"
              onClick={() => setConfirmando(false)}
              className="py-3 rounded-xl border-2 border-neutral-300 text-neutral-500 text-[10px] font-black uppercase tracking-widest hover:border-neutral-950 hover:text-neutral-950 transition-all active:scale-[0.98]"
            >
              Cancelar
            </button>
            <button
              type="button"
              onClick={() => onCambiarEstado("inactivo")}
              disabled={cambiandoEstado}
              className="py-3 rounded-xl bg-red-500 text-white text-[10px] font-black uppercase tracking-widest hover:bg-red-600 transition-all shadow-md shadow-red-200 active:scale-[0.98] disabled:opacity-40"
            >
              {cambiandoEstado ? "Desactivando..." : "Aceptar"}
            </button>
          </div>
        </div>
      )}
    </div>
  );
};

export default SeccionEstado;
