// Iconos (morphicons) y componentes compartidos del módulo Empleados.
// Piel reconstruida: blanco/negro, botones gorditos y micro-animaciones con MorphIcon.
import { useState, useEffect, type ReactNode } from "react";
import { MorphIcon, type IconInput } from "morphicons/react";

// ── ICONOS (paths tipo Lucide sobre rejilla 24×24) ──────────────────────────
export const ICONO_USUARIOS: IconInput = "M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2 M9 7a4 4 0 1 0 0 8 4 4 0 0 0 0-8 M23 21v-2a4 4 0 0 0-3-3.87 M16 3.13a4 4 0 0 1 0 7.75";
export const ICONO_USUARIO: IconInput = "M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2 M12 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8z";
export const ICONO_RELOJ: IconInput = "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z M12 6v6l4 2";
export const ICONO_TARGET: IconInput = "M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20z M12 6a6 6 0 1 0 0 12 6 6 0 0 0 0-12z M12 10a2 2 0 1 0 0 4 2 2 0 0 0 0-4z";
export const ICONO_DOLAR: IconInput = "M12 1v22 M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6";
export const ICONO_TRENDING: IconInput = "M23 6l-9.5 9.5-5-5L1 18 M17 6h6v6";
export const ICONO_PREMIO: IconInput = "M8 21h8 M12 17v4 M7 4h10v4a5 5 0 0 1-10 0V4z M7 5H4a2 2 0 0 0 2 4h3 M17 5h3a2 2 0 0 1-2 4h-3";
export const ICONO_CHECK: IconInput = "M20 6 9 17l-5-5";
export const ICONO_MAS: IconInput = "M12 5v14M5 12h14";
export const ICONO_CERRAR: IconInput = "M18 6 6 18M6 6l12 12";
export const ICONO_FLECHA: IconInput = "M5 12h14M12 5l7 7-7 7";
export const ICONO_EDITAR: IconInput = "M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z";
export const ICONO_BORRAR: IconInput = "M3 6h18 M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6 M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2";
export const ICONO_CALENDARIO: IconInput = "M8 2v4 M16 2v4 M3 10h18 M5 4h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2z";
export const ICONO_OJO: IconInput = "M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z";
export const ICONO_OJO_OCULTO: IconInput = "M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19M14.12 14.12a3 3 0 1 1-4.24-4.24M1 1l22 22";
export const ICONO_ALERTA: IconInput = "M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z M12 9v4 M12 17h.01";

// ── BOTÓN GORDITO CON MORPH EN HOVER ────────────────────────────────────────
interface BotonAnimadoProps {
  children: ReactNode;
  icono: IconInput;
  iconoHover?: IconInput;
  className?: string;
  disabled?: boolean;
  onClick?: () => void;
  type?: "button" | "submit";
}

export const BotonAnimado = ({
  children,
  icono,
  iconoHover,
  className = "",
  disabled,
  onClick,
  type = "button",
}: BotonAnimadoProps) => {
  const [hover, setHover] = useState(false);
  const mostrar = hover && iconoHover ? iconoHover : icono;
  return (
    <button
      type={type}
      onClick={onClick}
      disabled={disabled}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      className={`inline-flex items-center justify-center gap-2.5 px-6 py-3.5 rounded-2xl text-[10px] font-black uppercase tracking-[0.2em] transition-all duration-200 disabled:opacity-30 disabled:cursor-not-allowed active:scale-[0.97] ${className}`}
    >
      <MorphIcon icon={mostrar} size={16} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
      {children}
    </button>
  );
};

// ── MORPH CONTINUO: al entrar el cursor morfea A↔B en loop mientras siga ahí ──
interface IconoMorphProps {
  icono: IconInput;
  iconoHover: IconInput;
  size?: number;
  strokeWidth?: number;
  className?: string;
  /** Cada cuánto (ms) alterna entre icono e iconoHover mientras hay hover. */
  intervalo?: number;
  /** Hover controlado desde el padre (todo el bloque). Si se pasa, ignora el suyo propio. */
  hover?: boolean;
}

export const IconoMorph = ({ icono, iconoHover, size = 16, strokeWidth = 2.2, className = "", intervalo = 520, hover: hoverExterno }: IconoMorphProps) => {
  const [selfHover, setSelfHover] = useState(false);
  const [mostrar, setMostrar] = useState(icono);
  const hover = hoverExterno ?? selfHover;

  // Al entrar el cursor: morfea al icono Hover y se mantiene alternando mientras siga ahí.
  useEffect(() => {
    if (!hover) {
      setMostrar(icono);
      return;
    }
    setMostrar(iconoHover);
    const timer = window.setInterval(() => {
      setMostrar((prev) => (prev === icono ? iconoHover : icono));
    }, intervalo);
    return () => window.clearInterval(timer);
  }, [hover, icono, iconoHover, intervalo]);

  return (
    <span
      onMouseEnter={hoverExterno === undefined ? () => setSelfHover(true) : undefined}
      onMouseLeave={hoverExterno === undefined ? () => setSelfHover(false) : undefined}
      className={`inline-flex items-center justify-center ${className}`}
    >
      <MorphIcon icon={mostrar} size={size} strokeWidth={strokeWidth} spring="smooth" reducedMotion="user" />
    </span>
  );
};

// ── MODAL BASE: overlay oscuro + tarjeta blanca redondeada ──────────────────
interface ModalShellProps {
  icono: IconInput;
  titulo: string;
  subtitulo?: string;
  ancho?: string;
  onClose: () => void;
  children: ReactNode;
}

export const ModalShell = ({
  icono,
  titulo,
  subtitulo,
  ancho = "max-w-lg",
  onClose,
  children,
}: ModalShellProps) => (
  <div
    className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4"
    onClick={onClose}
  >
    <div
      className={`bg-white rounded-[2rem] shadow-2xl w-full ${ancho} p-8 space-y-6 animate-in zoom-in-95 duration-200 max-h-[88vh] overflow-y-auto custom-scrollbar`}
      onClick={(e) => e.stopPropagation()}
    >
      <header className="text-center">
        <div className="mx-auto w-12 h-12 bg-neutral-950 text-neutral-50 rounded-2xl flex items-center justify-center">
          <MorphIcon icon={icono} size={20} strokeWidth={2.2} spring="smooth" />
        </div>
        <h2 className="text-lg font-black text-neutral-900 uppercase mt-4 tracking-tight">{titulo}</h2>
        <div className="h-0.5 w-8 bg-neutral-950 mx-auto mt-2 rounded-full" />
        {subtitulo && (
          <p className="text-[10px] font-bold text-neutral-400 uppercase tracking-widest mt-2">{subtitulo}</p>
        )}
      </header>
      {children}
    </div>
  </div>
);

// ── CAMPO DE FORMULARIO ─────────────────────────────────────────────────────
interface CampoProps {
  label: string;
  children: ReactNode;
}

export const Campo = ({ label, children }: CampoProps) => (
  <div className="space-y-1.5">
    <label className="text-[10px] font-black text-neutral-500 uppercase tracking-wider ml-1">{label}</label>
    {children}
  </div>
);

export const inputCls = "w-full px-4 py-3 rounded-xl bg-neutral-50 border border-neutral-100 text-sm font-bold text-neutral-900 focus:outline-none focus:border-neutral-950 focus:ring-4 focus:ring-neutral-950/5 transition-all";