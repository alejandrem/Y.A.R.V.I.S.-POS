// ═══════════════════════════════════════════════════════════════════════════
// MODAL EMPLEADOS — Alta y edición unificada de empleado en un solo paso.
// Tarea única: crear o actualizar un empleado completo con UNA sola llamada:
//   · Modo CREAR (sin `empleado`): guarda via guardar_empleado. La contraseña
//     es obligatoria y el backend rechaza duplicadas entre empleados porque
//     el login es solo por clave.
//   · Modo EDITAR (con `empleado`): abre la misma paleta con los datos ya
//     precargados y guarda via editar_empleado. La contraseña es OPCIONAL
//     (vacía = no cambiar) y se reemplazan los bloques de horario completos.
//   · Horarios MÚLTIPLES: cada bloque tiene sus propios días (chips L-D) y
//     rango entrada/salida — ej: L,X,J,V 8-17 y S,D 8-12. Un día no puede
//     repetirse entre bloques.
//   · Pago SEMANAL con proyección en vivo (día/hora/mes se calculan solos).
// ═══════════════════════════════════════════════════════════════════════════

import { useState } from "react";
import { MorphIcon } from "morphicons/react";
import {
  ModalShell, Campo, inputCls,
  ICONO_USUARIO, ICONO_OJO, ICONO_OJO_OCULTO, ICONO_CHECK, ICONO_EDITAR,
} from "../../../components/ui";
import { notificarError, notificarExito } from "../../../components/notificaciones";
import { reportarError } from "../../../services/tauri";
import { guardarEmpleado, editarEmpleado, cambiarEstadoEmpleado } from "../../../services/empleados";
import SelectorHorarios from "./componentes/selector-horarios";
import CampoSalario from "./componentes/campo-salario";
import SeccionEstado from "./componentes/seccion-estado";
import { useBloquesHorario } from "./utilidades/use-bloques-horario";
import type { EmpleadoEditable } from "./utilidades/horario-empleado";

interface ModalEmpleadosProps {
  onClose: () => void;
  onSaved: () => void;
  /** Si viene, el modal opera en modo edición con datos precargados. */
  empleado?: EmpleadoEditable;
}

const ModalEmpleados = ({ onClose, onSaved, empleado }: ModalEmpleadosProps) => {
  const modoEdicion = !!empleado;
  const [name, setName] = useState(empleado?.nombre ?? "");
  const [pass, setPass] = useState("");
  const [confirmPass, setConfirmPass] = useState("");
  const [showPass, setShowPass] = useState(false);
  const {
    bloques,
    diasSemana,
    horasTotales,
    diasOcupadosEn,
    toggleDia,
    setBloque,
    agregarBloque,
    eliminarBloque,
    validar: validarBloques,
    aEnvio: bloquesAEnvio,
  } = useBloquesHorario(
    empleado?.horarios.length
      ? empleado.horarios.map((h) => ({ dias: [...h.dias], inicio: h.hora_inicio, fin: h.hora_fin }))
      : undefined,
  );
  const [salarioSemanal, setSalarioSemanal] = useState(empleado?.salario_semanal ?? 0);
  const [guardando, setGuardando] = useState(false);
  const [estadoActual, setEstadoActual] = useState(empleado?.estado ?? "activo");
  const [cambiandoEstado, setCambiandoEstado] = useState(false);
  const [confirmarDesactivar, setConfirmarDesactivar] = useState(false);

  const cambiarEstado = async (nuevoEstado: string) => {
    if (!empleado) return;
    setCambiandoEstado(true);
    try {
      await cambiarEstadoEmpleado(empleado.id, nuevoEstado);
      setEstadoActual(nuevoEstado);
      setConfirmarDesactivar(false);
      notificarExito(`Empleado ${nuevoEstado === "activo" ? "activado" : "desactivado"}`);
      onSaved();
    } catch (error) {
      reportarError("No se pudo cambiar el estado del empleado", error);
    } finally {
      setCambiandoEstado(false);
    }
  };

  const handleSave = async () => {
    if (!name.trim()) {
      notificarError("El nombre es obligatorio");
      return;
    }
    // En edición la contraseña es opcional (vacía = no cambiar).
    if (!modoEdicion || pass || confirmPass) {
      if (pass.length < 6 || !/[A-Za-z]/.test(pass) || !/[0-9]/.test(pass)) {
        notificarError("La contraseña debe tener al menos 6 caracteres, con letras y números");
        return;
      }
      if (pass !== confirmPass) {
        notificarError("Las contraseñas no coinciden");
        return;
      }
    }
    const errorBloques = validarBloques();
    if (errorBloques) {
      notificarError(errorBloques);
      return;
    }
    setGuardando(true);
    try {
      const horarios = bloquesAEnvio();
      if (modoEdicion && empleado) {
        await editarEmpleado({
          empleadoId: empleado.id,
          nombre: name.trim(),
          salarioSemanal,
          horarios,
          nuevaPassword: pass || null,
        });
      } else {
        await guardarEmpleado({ name: name.trim(), pass, salarioSemanal, horarios });
      }
      notificarExito(modoEdicion ? "Empleado actualizado" : "Empleado guardado");
      onSaved();
      onClose();
    } catch (error) {
      reportarError("No se pudo guardar el empleado", error);
    } finally {
      setGuardando(false);
    }
  };

  return (
    <ModalShell
      icono={modoEdicion ? ICONO_EDITAR : ICONO_USUARIO}
      titulo={modoEdicion ? "Editar Empleado" : "Nuevo Empleado"}
      subtitulo={modoEdicion ? empleado?.nombre : "Registro completo en un solo paso"}
      onClose={onClose}
      ancho="max-w-lg"
    >
      <div className="space-y-5">
        {/* ── ACCESO ─────────────────────────────────────────────── */}
        <div className="space-y-4">
          <Campo label="Nombre del Empleado">
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Ej. Peter Parker"
              autoFocus
              className={inputCls}
            />
          </Campo>

          <Campo label={modoEdicion ? "Nueva Contraseña (opcional)" : "Contraseña"}>
            <div className="relative">
              <input
                type={showPass ? "text" : "password"}
                value={pass}
                onChange={(e) => setPass(e.target.value)}
                placeholder={modoEdicion ? "Vacío = no cambiar" : "••••••••"}
                className={`${inputCls} pr-12`}
              />
              <button
                type="button"
                onClick={() => setShowPass(!showPass)}
                aria-label={showPass ? "Ocultar contraseña" : "Mostrar contraseña"}
                className="absolute right-3 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-neutral-950 transition-colors"
              >
                <MorphIcon icon={showPass ? ICONO_OJO_OCULTO : ICONO_OJO} size={17} strokeWidth={2} spring="snappy" reducedMotion="user" />
              </button>
            </div>
          </Campo>

          <Campo label={modoEdicion ? "Confirmar Nueva Contraseña" : "Confirmar Contraseña"}>
            <input
              type={showPass ? "text" : "password"}
              value={confirmPass}
              onChange={(e) => setConfirmPass(e.target.value)}
              placeholder="••••••••"
              className={`${inputCls} ${confirmPass && confirmPass === pass ? "ring-4 ring-emerald-500/10 border-emerald-400" : ""}`}
            />
          </Campo>
        </div>

        {/* ── HORARIOS DE TRABAJO (múltiples bloques) ────────────── */}
        <SelectorHorarios
          bloques={bloques}
          diasSemana={diasSemana}
          diasOcupadosEn={diasOcupadosEn}
          onToggleDia={toggleDia}
          onSetBloque={setBloque}
          onEliminar={eliminarBloque}
          onAgregar={agregarBloque}
        />

        {/* ── PAGO SEMANAL ───────────────────────────────────────── */}
        <CampoSalario
          salarioSemanal={salarioSemanal}
          onChange={setSalarioSemanal}
          diasSemana={diasSemana}
          horasTotales={horasTotales}
          totalBloques={bloques.length}
        />

        {/* ── ESTADO DEL EMPLEADO (solo edición) ─────────────────── */}
        {modoEdicion && (
          <SeccionEstado
            nombreEmpleado={empleado?.nombre}
            estadoActual={estadoActual}
            confirmando={confirmarDesactivar}
            setConfirmando={setConfirmarDesactivar}
            cambiandoEstado={cambiandoEstado}
            onCambiarEstado={cambiarEstado}
          />
        )}
      </div>

      <div className="pt-2 space-y-2">
        <button
          onClick={handleSave}
          disabled={guardando}
          className="w-full inline-flex items-center justify-center gap-2.5 py-4 rounded-xl bg-neutral-950 text-neutral-50 text-xs font-black uppercase tracking-[0.2em] hover:bg-neutral-800 transition-all shadow-xl shadow-neutral-200 active:scale-[0.98] disabled:opacity-40"
        >
          <MorphIcon icon={ICONO_CHECK} size={16} strokeWidth={2.5} spring="snappy" />
          {guardando ? "Guardando..." : modoEdicion ? "Guardar Cambios" : "Registrar Empleado"}
        </button>
        <button
          onClick={onClose}
          className="w-full py-3 text-[10px] font-black text-neutral-400 uppercase tracking-widest hover:text-neutral-900 transition-colors"
        >
          Cancelar
        </button>
      </div>
    </ModalShell>
  );
};

export default ModalEmpleados;
