// Tarjeta de resumen del panel: el hover de TODO el bloque
// activa el loop del morphicon.
import { useState, type ReactNode } from "react";
import { type IconInput } from "morphicons/react";
import { IconoMorph } from "../../../../components/ui";

export interface TarjetaResumenProps {
  icono: IconInput;
  iconoHover: IconInput;
  label: string;
  valor: string;
  oscura?: boolean;
  children?: ReactNode;
}

export const TarjetaResumen = ({ icono, iconoHover, label, valor, oscura = false, children }: TarjetaResumenProps) => {
  const [hover, setHover] = useState(false);
  return (
    <div
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      className={`rounded-[2rem] p-5 sm:p-6 border transition-colors duration-200 ${
        oscura ? "bg-neutral-950 text-neutral-50 border-neutral-950" : "bg-white text-neutral-900 border-neutral-200"
      }`}
    >
      <div
        className={`w-10 h-10 rounded-xl flex items-center justify-center mb-3 transition-colors duration-200 ${
          oscura ? "bg-white/10 text-neutral-50" : "bg-neutral-950 text-neutral-50"
        }`}
      >
        <IconoMorph icono={icono} iconoHover={iconoHover} size={16} strokeWidth={2.2} hover={hover} />
      </div>
      <p className={`text-[9px] font-black uppercase tracking-widest ${oscura ? "opacity-70" : "text-neutral-400"}`}>
        {label}
      </p>
      <p className="text-2xl font-black mt-1">{valor}</p>
      {children}
    </div>
  );
};
