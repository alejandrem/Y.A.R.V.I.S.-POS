import { MorphIcon, type IconInput } from "morphicons/react";
import { useState } from "react";

interface SeparadorProps {
  label: string;
  icon: IconInput;
  left: string; // ej "58%"
  activo?: boolean;
  onClick: () => void;
}

const Separador = ({ label, icon, left, activo, onClick }: SeparadorProps) => {
  const [hover, setHover] = useState(false);

  return (
    <button
      onClick={(e) => {
        e.stopPropagation();
        onClick();
      }}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      style={{ left }}
      className={`absolute -top-1 w-8 h-11 rounded-b-lg shadow-md flex flex-col items-center justify-end pb-1.5 z-20 transition-all duration-200 cursor-pointer
        ${activo ? "bg-neutral-900 h-12 -top-1.5" : "bg-neutral-900 hover:h-12 hover:-top-1.5"}`}
      title={label}
      aria-label={label}
    >
      <MorphIcon icon={icon} size={14} strokeWidth={2.2} spring="smooth" className="text-white" />
      {/* tooltip al hover */}
      {hover && (
        <span className="absolute -top-8 left-1/2 -translate-x-1/2 bg-neutral-900 text-white font-mono text-[9px] font-black tracking-widest px-2 py-1 rounded whitespace-nowrap pointer-events-none">
          {label}
        </span>
      )}
      <div className="w-4 h-[2px] bg-white/60 rounded-full mt-1" />
    </button>
  );
};

export default Separador;
