// ═══════════════════════════════════════════════════════════════════════════
// USE BLOQUES HORARIO — Estado compartido de bloques de horario semanal.
//
// Lo usan ModalEmpleados (alta/edición) y PrimerInicio (alta inicial) para
// no duplicar la lógica: días sin repetir entre bloques, totales derivados
// y validación con los mismos mensajes. Presentación en SelectorHorarios.
// ═══════════════════════════════════════════════════════════════════════════

import { useState } from "react";
import { bloqueNuevo, bloqueVacio, calcularHorasTotales, type Bloque } from "./horario-empleado";
import type { HorarioEnvio } from "../../../../services/empleados";

export const useBloquesHorario = (inicial?: Bloque[]) => {
  const [bloques, setBloques] = useState<Bloque[]>(inicial ?? [bloqueVacio()]);

  const diasSemana = new Set(bloques.flatMap((b) => b.dias)).size;
  const horasTotales = calcularHorasTotales(bloques);

  const diasOcupadosEn = (idxBloque: number) =>
    new Set(bloques.filter((_, i) => i !== idxBloque).flatMap((b) => b.dias));

  const toggleDia = (idxBloque: number, dia: number) =>
    setBloques((prev) => {
      // Ocupados calculados del `prev` del updater, no del closure: con
      // clicks rápidos seguidos el closure queda stale y metía duplicados.
      const ocupados = new Set(prev.filter((_, i) => i !== idxBloque).flatMap((b) => b.dias));
      return prev.map((b, i) => {
        if (i !== idxBloque) return b;
        return b.dias.includes(dia)
          ? { ...b, dias: b.dias.filter((d) => d !== dia) }
          : ocupados.has(dia)
            ? b // el día ya pertenece a otro bloque: ignorar
            : { ...b, dias: [...b.dias, dia].sort() };
      });
    });

  const setBloque = (idxBloque: number, patch: Partial<Bloque>) =>
    setBloques((prev) => prev.map((b, i) => (i === idxBloque ? { ...b, ...patch } : b)));

  const agregarBloque = () => setBloques((prev) => [...prev, bloqueNuevo()]);

  const eliminarBloque = (idx: number) =>
    setBloques((prev) => prev.filter((_, i) => i !== idx));

  const restablecer = () => setBloques([bloqueVacio()]);

  /** Mensaje de error listo para notificarError, o null si todo cuadra. */
  const validar = (): string | null => {
    for (let i = 0; i < bloques.length; i++) {
      if (bloques[i].dias.length === 0) {
        return `El horario #${i + 1} no tiene días seleccionados`;
      }
      if (!bloques[i].inicio || !bloques[i].fin) {
        return `Define la hora de entrada y salida del horario #${i + 1}`;
      }
    }
    if (diasSemana === 0) {
      return "Selecciona al menos un día de trabajo";
    }
    return null;
  };

  const aEnvio = (): HorarioEnvio[] =>
    bloques.map((b) => ({ dias: b.dias, horaInicio: b.inicio, horaFin: b.fin }));

  return {
    bloques,
    diasSemana,
    horasTotales,
    diasOcupadosEn,
    toggleDia,
    setBloque,
    agregarBloque,
    eliminarBloque,
    restablecer,
    validar,
    aEnvio,
  };
};
