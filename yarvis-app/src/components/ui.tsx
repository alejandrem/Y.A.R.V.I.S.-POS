// ─────────────────────────────────────────────────────────────────────────────
// UI COMPARTIDA (componentes primitivos de toda la app)
// ─────────────────────────────────────────────────────────────────────────────
// Aquí viven los componentes genéricos reutilizables (botones, modales, inputs,
// morph helpers) que cualquier módulo importa. NO duplicar en carpetas de
// features: si una primitiva se va a usar en dos lados, va aquí.
// Los iconos morpheables se re-exportan desde src/icons.ts (única fuente).
// ─────────────────────────────────────────────────────────────────────────────
import { useState, useEffect, type ReactNode } from "react";
import { MorphIcon, type IconInput } from "morphicons/react";

// Re-export de la única fuente de iconos para importar cómodo desde aquí.
export {
  ICONO_CHECK,
  ICONO_CHECK_CIRCULO,
  ICONO_MAS,
  ICONO_MAS_CIRCULO,
  ICONO_RESTA,
  ICONO_RESTA_CIRCULO,
  ICONO_EQUIS,
  ICONO_CERRAR,
  ICONO_FLECHA,
  ICONO_HISTORIAL,
  ICONO_RELOJ,
  ICONO_CALENDARIO,
  ICONO_CAMPANA,
  ICONO_ALERTA,
  ICONO_ALERTA_CIRCULO,
  ICONO_INFO,
  ICONO_AYUDA,
  ICONO_MENU,
  ICONO_BUSCAR,
  ICONO_FILTRO,
  ICONO_ETIQUETA,
  ICONO_CODIGO,
  ICONO_ENLACE,
  ICONO_INICIO,
  ICONO_UBICACION,
  ICONO_MUNDO,
  ICONO_NUBE,
  ICONO_CORREO,
  ICONO_TELEFONO,
  ICONO_DOCUMENTO,
  ICONO_DOCUMENTO_NUEVO,
  ICONO_CARPETA,
  ICONO_LIBRO,
  ICONO_FOTO,
  ICONO_CAMARA,
  ICONO_IMPRESORA,
  ICONO_DESCARGAR,
  ICONO_SUBIR,
  ICONO_DOLAR,
  ICONO_TRENDING,
  ICONO_GRAFICA,
  ICONO_PREMIO,
  ICONO_TROFEO,
  ICONO_ESTRELLA,
  ICONO_CORAZON,
  ICONO_BILLETE,
  ICONO_TARJETA,
  ICONO_PORCENTAJE,
  ICONO_BOLSA,
  ICONO_CARRITO,
  ICONO_CALCULADORA,
  ICONO_CAJA,
  ICONO_CAMION,
  ICONO_BASE_DATOS,
  ICONO_ESCANER,
  ICONO_CODIGO_BARRAS,
  ICONO_CELULAR,
  ICONO_PANTALLA,
  ICONO_SOL,
  ICONO_LUNA,
  ICONO_FUEGO,
  ICONO_USUARIOS,
  ICONO_USUARIO,
  ICONO_OJO,
  ICONO_OJO_OCULTO,
  ICONO_APROBADO,
  ICONO_TARGET,
  ICONO_CANDADO,
  ICONO_EDITAR,
  ICONO_BORRAR,
  ICONO_REINICIAR,
  ICONO_TIENDA,
  ICONO_REGALO,
  ICONO_ENVIAR,
  ICONO_PAUSA,
  ICONO_ROBOT,
  ICONO_ENGRANAJE,
} from "../icons";

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