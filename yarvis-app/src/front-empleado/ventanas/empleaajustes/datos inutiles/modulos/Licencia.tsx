import { ICONO_DOCUMENTO } from "../../../../../components/ui";

export const licenciaSeparador = {
  id: "licencia",
  label: "LICENCIA",
  icon: ICONO_DOCUMENTO,
  left: "93%",
};

export const LicenciaIzq = () => (
  <div className="flex-1 bg-white border-r-[3px] border-neutral-900 p-10 flex flex-col relative overflow-hidden">
    <div className="absolute top-8 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="absolute top-12 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="flex-1 flex flex-col items-center justify-center text-center">
      <p className="font-mono text-[11px] font-black tracking-[0.35em] text-neutral-900 border-2 border-neutral-900 px-3 py-1.5 rounded-lg">
        MODULO 08 // LEGAL
      </p>
      <h3 className="font-mono text-[38px] font-black tracking-[0.12em] text-neutral-900 leading-none mt-6">LICENCIA</h3>
      <div className="w-20 h-[4px] bg-neutral-900 mt-6" />
      <p className="font-mono text-[13px] font-black tracking-[0.12em] text-neutral-900 mt-4">GPL V3</p>
      <p className="font-mono text-[11px] font-bold tracking-widest text-neutral-500 mt-2 max-w-[320px] leading-relaxed">
        Libre, copyleft fuerte.
        <br />
        Siempre será libre.
      </p>
      <div className="mt-6 flex gap-2">
        <span className="font-mono text-[8px] font-black tracking-widest bg-neutral-900 text-white px-2 py-1 rounded">PAG. 17 — 18</span>
      </div>
    </div>
    <p className="font-mono text-[11px] font-black tracking-widest text-neutral-400 text-center">— 17 —</p>
    <div className="absolute bottom-0 left-0 w-6 h-6 border-t-[2px] border-r-[2px] border-neutral-900/20 rounded-tr-xl" />
  </div>
);

export const LicenciaDer = () => (
  <div className="flex-1 bg-white p-8 flex flex-col relative overflow-hidden">
    <div className="absolute top-8 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="absolute top-12 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="flex-1 flex flex-col justify-center gap-3 mt-2">
      <div className="bg-white border-2 border-neutral-900 rounded-2xl p-4">
        <p className="font-mono text-[10px] font-black tracking-widest text-neutral-900 flex items-center gap-2">
          <span className="w-2 h-2 bg-neutral-900 rounded-full" />
          QUE PERMITE
        </p>
        <ul className="mt-2 space-y-1.5 font-mono text-[9px] font-bold text-neutral-600 leading-relaxed list-disc list-inside">
          <li>Usar el software para cualquier propósito, incluso comercial.</li>
          <li>Estudiar y modificar el código fuente.</li>
          <li>Distribuir copias originales o modificadas.</li>
        </ul>
      </div>
      <div className="bg-white border-2 border-neutral-900 rounded-2xl p-4">
        <p className="font-mono text-[10px] font-black tracking-widest text-neutral-900 flex items-center gap-2">
          <span className="w-2 h-2 bg-neutral-900 rounded-full" />
          CONDICION
        </p>
        <p className="font-mono text-[9px] font-bold text-neutral-600 leading-relaxed mt-2">
          Si distribuyes, debes hacerlo bajo GPLv3 y entregar el código fuente. El copyleft protege que siga libre.
        </p>
      </div>
      <div className="bg-neutral-900 rounded-2xl p-4 border-2 border-neutral-900">
        <p className="font-mono text-[10px] font-black tracking-widest text-white flex items-center gap-2">
          <span className="w-2 h-2 bg-white rounded-full" />
          CON EL CODIGO Y EL SOFTWARE
        </p>
        <p className="font-mono text-[9px] font-bold text-white/60 leading-relaxed mt-2">
          Puedes vender el POS, instalarlo en tiendas y cobrar por soporte, pero no puedes cerrarlo. Las mejoras deben volver a la comunidad.
        </p>
      </div>
    </div>
    <p className="font-mono text-[11px] font-black tracking-widest text-neutral-400 text-center mt-2">— 18 —</p>
    <div className="absolute bottom-0 right-0 w-6 h-6 border-t-[2px] border-l-[2px] border-neutral-900/20 rounded-tl-xl" />
  </div>
);
