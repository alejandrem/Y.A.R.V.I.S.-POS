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
import { invoke } from "@tauri-apps/api/core";
import { MorphIcon } from "morphicons/react";
import {
  ModalShell, Campo, inputCls,
  ICONO_USUARIO, ICONO_OJO, ICONO_OJO_OCULTO, ICONO_CHECK, ICONO_EDITAR,
} from "../../../components/ui";
import SelectorHorarios from "./componentes/selector-horarios";
import CampoSalario from "./componentes/campo-salario";
import SeccionEstado from "./componentes/seccion-estado";
import { bloqueVacio, calcularHorasTotales, type Bloque, type EmpleadoEditable } from "./utilidades/horario-empleado";

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
  const [bloques, setBloques] = useState<Bloque[]>(
    empleado?.horarios.length
      ? empleado.horarios.map((h) => ({ dias: [...h.dias], inicio: h.hora_inicio, fin: h.hora_fin }))
      : [bloqueVacio()],
  );
  const [salarioSemanal, setSalarioSemanal] = useState(empleado?.salario_semanal ?? 0);
  const [guardando, setGuardando] = useState(false);
  const [estadoActual, setEstadoActual] = useState(empleado?.estado ?? "activo");
  const [cambiandoEstado, setCambiandoEstado] = useState(false);
  const [confirmarDesactivar, setConfirmarDesactivar] = useState(false);
  const diasOcupadosEn = (idxBloque: number) =>
    new Set(bloques.filter((_, i) => i !== idxBloque).flatMap((b) => b.dias));

  const toggleDia = (idxBloque: number, dia: number) =>
    setBloques((prev) =>
      prev.map((b, i) => {
        if (i !== idxBloque) return b;
        return b.dias.includes(dia)
          ? { ...b, dias: b.dias.filter((d) => d !== dia) }
          : diasOcupadosEn(idxBloque).has(dia)
            ? b // el día ya pertenece a otro bloque: ignorar
            : { ...b, dias: [...b.dias, dia].sort() };
      }),
    );

  const setBloque = (idxBloque: number, patch: Partial<Bloque>) =>
    setBloques((prev) => prev.map((b, i) => (i === idxBloque ? { ...b, ...patch } : b)));

  // Totales derivados
  const diasSemana = new Set(bloques.flatMap((b) => b.dias)).size;
  const horasTotales = calcularHorasTotales(bloques);

  const cambiarEstado = async (nuevoEstado: string) => {
    if (!empleado) return;
    setCambiandoEstado(true);
    try {
      await invoke("set_estado_empleado", { empleadoId: empleado.id, estado: nuevoEstado });
      setEstadoActual(nuevoEstado);
      setConfirmarDesactivar(false);
      onSaved();
    } catch (error) {
      console.error("Error al cambiar estado del empleado:", error);
      alert(String(error));
    } finally {
      setCambiandoEstado(false);
    }
  };

  const handleSave = async () => {
    if (!name.trim()) {
      alert("El nombre es obligatorio");
      return;
    }
    // En edición la contraseña es opcional (vacía = no cambiar).
    if (!modoEdicion || pass || confirmPass) {
      if (pass.length < 6 || !/[A-Za-z]/.test(pass) || !/[0-9]/.test(pass)) {
        alert("La contraseña debe tener al menos 6 caracteres, con letras y números");
        return;
      }
      if (pass !== confirmPass) {
        alert("Las contraseñas no coinciden");
        return;
      }
    }
    for (let i = 0; i < bloques.length; i++) {
      if (bloques[i].dias.length === 0) {
        alert(`El horario #${i + 1} no tiene días seleccionados`);
        return;
      }
      if (!bloques[i].inicio || !bloques[i].fin) {
        alert(`Define la hora de entrada y salida del horario #${i + 1}`);
        return;
      }
    }
    if (diasSemana === 0) {
      alert("Selecciona al menos un día de trabajo");
      return;
    }
    setGuardando(true);
    try {
      const horarios = bloques.map((b) => ({ dias: b.dias, horaInicio: b.inicio, horaFin: b.fin }));
      if (modoEdicion && empleado) {
        await invoke("editar_empleado", {
          empleadoId: empleado.id,
          nombre: name.trim(),
          salarioSemanal,
          horarios,
          nuevaPassword: pass || null,
        });
      } else {
        await invoke("guardar_empleado", { name: name.trim(), pass, salarioSemanal, horarios });
      }
      onSaved();
      onClose();
    } catch (error) {
      console.error("Error al guardar empleado:", error);
      alert(String(error));
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
          onEliminar={(idx) => setBloques((prev) => prev.filter((_, i) => i !== idx))}
          onAgregar={() => setBloques((prev) => [...prev, bloqueVacio()])}
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
