// Hook auxiliar para secuencias de iconos morphing (ej. + → pausa → check → +).
// Se usa en el botón "Nuevo chat", en enviar/limpiar para animar el cambio de ícono.
import { useEffect, useRef, useState } from "react";
import type { IconInput } from "morphicons/react";

/**
 * Manejo de secuencias de iconos morphing (p.ej. + → pausa → check → +).
 * `play` recorre pasos con retardo; `jump` salta directo sin animar.
 * Cualquier llamada cancela los pasos pendientes anteriores.
 */
export function useIconSequence(initial: IconInput) {
  const [icon, setIcon] = useState<IconInput>(initial);
  const timeoutsRef = useRef<number[]>([]);

  const stop = () => {
    timeoutsRef.current.forEach(window.clearTimeout);
    timeoutsRef.current = [];
  };

  const play = (steps: { icon: IconInput; delay: number }[]) => {
    stop();
    timeoutsRef.current = steps.map((step) => window.setTimeout(() => setIcon(step.icon), step.delay));
  };

  const jump = (value: IconInput) => {
    stop();
    setIcon(value);
  };

  useEffect(() => () => stop(), []);

  return { icon, play, jump, stop };
}