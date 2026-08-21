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
  ICONO_USUARIO, ICONO_OJO, ICONO_OJO_OCULTO, ICONO_CHECK,
  ICONO_RELOJ, ICONO_DOLAR, ICONO_MAS, ICONO_BORRAR, ICONO_EDITAR,
  ICONO_ALERTA, ICONO_CERRAR,
} from "../../../components/ui";

interface EmpleadoEditable {
  id: number;
  nombre: string;
  estado: string;
  salario_semanal: number;
  horarios: { dias: number[]; hora_inicio: string; hora_fin: string }[];
}

interface ModalEmpleadosProps {
  onClose: () => void;
  onSaved: () => void;
  /** Si viene, el modal opera en modo edición con datos precargados. */
  empleado?: EmpleadoEditable;
}

interface Bloque {
  dias: number[];
  inicio: string;
  fin: string;
}

const DIAS = [
  { corto: "L", label: "Lunes" },
  { corto: "M", label: "Martes" },
  { corto: "X", label: "Miércoles" },
  { corto: "J", label: "Jueves" },
  { corto: "V", label: "Viernes" },
  { corto: "S", label: "Sábado" },
  { corto: "D", label: "Domingo" },
];

const bloqueVacio = (): Bloque => ({ dias: [0, 1, 2, 3, 4], inicio: "09:00", fin: "17:00" });

const detectTurno = (inicio: string) => {
  if (!inicio) return "";
  const h = parseInt(inicio.split(":")[0], 10);
  if (h >= 5 && h < 12) return "Matutino";
  if (h >= 12 && h < 19) return "Vespertino";
  return "Nocturno";
};


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
  const [confirmarDesactivar, setConfirmarDesactivar] = useState(false);
  const [cambiandoEstado, setCambiandoEstado] = useState(false);

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
  const horasTotales = bloques.reduce((acc, b) => {
    if (!b.inicio || !b.fin) return acc;
    const [ih, im] = b.inicio.split(":").map(Number);
    const [fh, fm] = b.fin.split(":").map(Number);
    let mins = fh * 60 + fm - (ih * 60 + im);
    if (mins <= 0) mins += 24 * 60; // turno nocturno cruza medianoche
    return acc + mins / 60;
  }, 0);
  const salarioDiario = diasSemana > 0 ? salarioSemanal / diasSemana : 0;
  const horasPorDia = diasSemana > 0 ? horasTotales / diasSemana : 0;
  const salarioHora = horasTotales > 0 ? salarioSemanal / horasTotales : 0;
  const salarioMensual = salarioSemanal * 4.33;

  const proyeccion = [
    { label: "× Hora", valor: salarioHora },
    { label: "× Día", valor: salarioDiario },
    { label: "× Semana", valor: salarioSemanal },
    { label: "× Mes", valor: salarioMensual },
  ];

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
        <div className="bg-neutral-50 rounded-2xl p-4 space-y-4">
          <p className="flex items-center gap-2 text-[10px] font-black text-neutral-500 uppercase tracking-widest">
            <MorphIcon icon={ICONO_RELOJ} size={14} strokeWidth={2.4} spring="smooth" />
            Horarios de trabajo
          </p>

          {bloques.map((b, idx) => (
            <div key={idx} className="bg-white rounded-xl p-3.5 border border-neutral-100 space-y-3 relative">
              <div className="flex items-center justify-between">
                <span className="flex items-center gap-2">
                  <span className="px-2 py-0.5 bg-neutral-950 text-white text-[9px] font-black rounded-lg">#{idx + 1}</span>
                  {detectTurno(b.inicio) && (
                    <span className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">Turno {detectTurno(b.inicio)}</span>
                  )}
                </span>
                {bloques.length > 1 && (
                  <button
                    type="button"
                    onClick={() => setBloques((prev) => prev.filter((_, i) => i !== idx))}
                    aria-label={`Eliminar horario ${idx + 1}`}
                    className="text-neutral-300 hover:text-red-500 transition-colors"
                  >
                    <MorphIcon icon={ICONO_BORRAR} size={14} strokeWidth={2.2} spring="snappy" />
                  </button>
                )}
              </div>

              <div className="grid grid-cols-2 gap-3">
                <Campo label="Entrada">
                  <input type="time" value={b.inicio} onChange={(e) => setBloque(idx, { inicio: e.target.value })} className={inputCls} />
                </Campo>
                <Campo label="Salida">
                  <input type="time" value={b.fin} onChange={(e) => setBloque(idx, { fin: e.target.value })} className={inputCls} />
                </Campo>
              </div>

              <div>
                <p className="text-[10px] font-black text-neutral-500 uppercase tracking-widest mb-2">Días</p>
                <div className="grid grid-cols-7 gap-1.5">
                  {DIAS.map((d, dia) => {
                    const activo = b.dias.includes(dia);
                    const ocupado = !activo && diasOcupadosEn(idx).has(dia);
                    return (
                      <button
                        key={dia}
                        type="button"
                        title={ocupado ? `${d.label}: ya está en otro horario` : d.label}
                        onClick={() => toggleDia(idx, dia)}
                        disabled={ocupado}
                        className={`py-2 rounded-xl text-[11px] font-black transition-all active:scale-[0.92] ${
                          activo
                            ? "bg-neutral-950 text-neutral-50 shadow-md"
                            : ocupado
                              ? "bg-neutral-100 text-neutral-200 cursor-not-allowed line-through"
                              : "bg-neutral-50 text-neutral-400 border border-neutral-200 hover:border-neutral-400 hover:text-neutral-700"
                        }`}
                      >
                        {d.corto}
                      </button>
                    );
                  })}
                </div>
              </div>
            </div>
          ))}

          <button
            type="button"
            onClick={() => setBloques((prev) => [...prev, bloqueVacio()])}
            disabled={diasSemana >= 7}
            className="w-full inline-flex items-center justify-center gap-2 py-2.5 rounded-xl border-2 border-dashed border-neutral-300 text-neutral-400 text-[10px] font-black uppercase tracking-widest hover:border-neutral-950 hover:text-neutral-950 transition-all disabled:opacity-30 disabled:hover:border-neutral-300 disabled:hover:text-neutral-400"
          >
            <MorphIcon icon={ICONO_MAS} size={14} strokeWidth={2.5} spring="snappy" />
            Agregar otro horario
            {diasSemana >= 7 && " (todos los días asignados)"}
          </button>
        </div>

        {/* ── PAGO SEMANAL ───────────────────────────────────────── */}
        <div className="bg-neutral-50 rounded-2xl p-4 space-y-3">
          <p className="flex items-center gap-2 text-[10px] font-black text-neutral-500 uppercase tracking-widest">
            <MorphIcon icon={ICONO_DOLAR} size={14} strokeWidth={2.4} spring="smooth" />
            Pago semanal
          </p>
          <Campo label="Pago por semana ($)">
            <input
              type="number"
              min={0}
              step={50}
              value={salarioSemanal || ""}
              onChange={(e) => setSalarioSemanal(Math.max(0, Number(e.target.value)))}
              placeholder="0"
              className={inputCls}
            />
          </Campo>
          <div className="grid grid-cols-4 gap-2">
            {proyeccion.map((p) => (
              <div key={p.label} className="bg-white rounded-xl p-2.5 text-center border border-neutral-100">
                <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest">{p.label}</p>
                <p className="text-[13px] font-black text-neutral-900 mt-0.5">${p.valor.toFixed(2)}</p>
              </div>
            ))}
          </div>
          <p className="text-[8px] font-bold text-neutral-400 text-center">
            Basado en {horasPorDia.toFixed(1)}h/día · {diasSemana} días/semana · {bloques.length} {bloques.length === 1 ? "horario" : "horarios"}
          </p>
        </div>

        {/* ── ESTADO DEL EMPLEADO (solo edición) ─────────────────── */}
        {modoEdicion && (
          <div className={`rounded-2xl p-4 space-y-3 border-2 ${estadoActual === "inactivo" ? "border-red-200 bg-red-50/60" : "border-neutral-200 bg-white"}`}>
            <p className="flex items-center gap-2 text-[10px] font-black text-neutral-500 uppercase tracking-widest">
              <MorphIcon icon={ICONO_ALERTA} size={14} strokeWidth={2.4} spring="smooth" className={estadoActual === "inactivo" ? "text-red-500" : ""} />
              Estado ·{" "}
              <span className={estadoActual === "inactivo" ? "text-red-500" : "text-emerald-600"}>
                {estadoActual === "inactivo" ? "Inactivo" : "Activo"}
              </span>
            </p>

            {!confirmarDesactivar ? (
              <>
                <button
                  type="button"
                  onClick={() => setConfirmarDesactivar(true)}
                  disabled={estadoActual === "inactivo"}
                  className="w-full inline-flex items-center justify-center gap-2 py-3 rounded-xl bg-red-500 text-white text-[10px] font-black uppercase tracking-[0.15em] hover:bg-red-600 transition-all shadow-lg shadow-red-200 active:scale-[0.98] disabled:opacity-30 disabled:cursor-not-allowed"
                >
                  <MorphIcon icon={ICONO_CERRAR} size={14} strokeWidth={2.5} spring="snappy" />
                  Desactivar Empleado
                </button>
                {estadoActual === "inactivo" && (
                  <button
                    type="button"
                    onClick={() => cambiarEstado("activo")}
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
                    ¿Desactivar a {empleado?.nombre}?
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
                    onClick={() => setConfirmarDesactivar(false)}
                    className="py-3 rounded-xl border-2 border-neutral-300 text-neutral-500 text-[10px] font-black uppercase tracking-widest hover:border-neutral-950 hover:text-neutral-950 transition-all active:scale-[0.98]"
                  >
                    Cancelar
                  </button>
                  <button
                    type="button"
                    onClick={() => cambiarEstado("inactivo")}
                    disabled={cambiandoEstado}
                    className="py-3 rounded-xl bg-red-500 text-white text-[10px] font-black uppercase tracking-widest hover:bg-red-600 transition-all shadow-md shadow-red-200 active:scale-[0.98] disabled:opacity-40"
                  >
                    {cambiandoEstado ? "Desactivando..." : "Aceptar"}
                  </button>
                </div>
              </div>
            )}
          </div>
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
