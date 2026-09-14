import { ICONO_DOCUMENTO } from "../../../../../components/ui";

export const ticketsSeparador = {
  id: "tickets",
  label: "TICKETS",
  icon: ICONO_DOCUMENTO,
  left: "58%",
};

export const TicketsIzq = () => (
  <div className="flex-1 bg-white border-r-[3px] border-neutral-900 p-10 flex flex-col relative overflow-hidden">
    <div className="absolute top-8 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="absolute top-12 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="flex-1 flex flex-col items-center justify-center text-center">
      <p className="font-mono text-[11px] font-black tracking-[0.35em] text-neutral-900 border-2 border-neutral-900 px-3 py-1.5 rounded-lg">
        MODULO 03 // HISTORIAL
      </p>
      <h3 className="font-mono text-[30px] font-black tracking-[0.10em] text-neutral-900 leading-none mt-6">TICKETS Y</h3>
      <h3 className="font-mono text-[30px] font-black tracking-[0.10em] text-neutral-900 leading-none">CORTES</h3>
      <div className="w-20 h-[4px] bg-neutral-900 mt-6" />
      <p className="font-mono text-[11px] font-bold tracking-widest text-neutral-500 mt-4 max-w-[320px] leading-relaxed">
        Cada venta con su ticket
        <br />y su corte.
      </p>
      <div className="mt-6 flex gap-2">
        <span className="font-mono text-[8px] font-black tracking-widest bg-neutral-900 text-white px-2 py-1 rounded">PAG. 07 — 08</span>
      </div>
    </div>
    <p className="font-mono text-[11px] font-black tracking-widest text-neutral-400 text-center">— 07 —</p>
    <div className="absolute bottom-0 left-0 w-6 h-6 border-t-[2px] border-r-[2px] border-neutral-900/20 rounded-tr-xl" />
  </div>
);

export const TicketsDer = () => (
  <div className="flex-1 bg-white p-10 flex flex-col relative overflow-hidden">
    <div className="absolute top-8 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="absolute top-12 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="flex-1 flex flex-col items-center justify-center text-center gap-6">
      <div className="w-20 h-20 bg-white border-[3px] border-neutral-900 rounded-2xl flex items-center justify-center">
        <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="black" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
          <path d="M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z" />
          <path d="M12 6v6l4 2" />
        </svg>
      </div>
      <div>
        <p className="font-mono text-[18px] font-black tracking-[0.2em] text-neutral-900">PROXIMAMENTE</p>
        <p className="font-mono text-[11px] font-bold tracking-widest text-neutral-500 mt-2 max-w-[280px] leading-relaxed">
          Este modulo esta vacio.
          <br />
          Pronto veras tus tickets y cortes aqui.
        </p>
      </div>
      <div className="border-2 border-dashed border-neutral-300 rounded-2xl px-4 py-3 bg-neutral-50">
        <p className="font-mono text-[9px] font-black tracking-widest text-neutral-400">MODULO EN CONSTRUCCION // TICKETS Y CORTES</p>
      </div>
    </div>
    <p className="font-mono text-[11px] font-black tracking-widest text-neutral-400 text-center mt-2">— 08 —</p>
    <div className="absolute bottom-0 right-0 w-6 h-6 border-t-[2px] border-l-[2px] border-neutral-900/20 rounded-tl-xl" />
  </div>
);
