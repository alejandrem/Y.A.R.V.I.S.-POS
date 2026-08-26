// Libro abierto - Datos Inutiles
// Manual del POS con animacion de hoja real, separadores por modulo y paginas con SVG
import { useState } from "react";
import Separador from "./componentes/Separador";
import { cobrarSeparador, CobrarIzq, CobrarDer } from "./modulos/Cobrar";
import { inventarioSeparador, InventarioIzq, InventarioDer } from "./modulos/Inventario";
import { ticketsSeparador, TicketsIzq, TicketsDer } from "./modulos/Tickets";
import { clientesSeparador, ClientesIzq, ClientesDer } from "./modulos/Clientes";
import { perfilSeparador, PerfilIzq, PerfilDer } from "./modulos/Perfil";
import { yarvisSeparador, YarvisIzq, YarvisDer } from "./modulos/Yarvis";
import { inutilSeparador, InutilIzq, InutilDer } from "./modulos/Inutil";
import { licenciaSeparador, LicenciaIzq, LicenciaDer } from "./modulos/Licencia";

type Spread = {
  id: string;
  left: React.ReactNode;
  right: React.ReactNode;
};

// Portada - Izquierda (titulo)
const PortadaIzq = () => (
  <div className="flex-1 bg-white border-r-[3px] border-neutral-900 p-10 flex flex-col relative overflow-hidden">
    <div className="absolute top-8 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="absolute top-12 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="flex-1 flex flex-col items-center justify-center text-center">
      <p className="font-mono text-[11px] font-black tracking-[0.35em] text-neutral-900 border-2 border-neutral-900 px-3 py-1.5 rounded-lg">
        Y.A.R.V.I.S. POS // v0.1.0
      </p>
      <h3 className="font-mono text-[44px] sm:text-[52px] font-black tracking-[0.12em] text-neutral-900 leading-none mt-6">DATOS</h3>
      <h3 className="font-mono text-[44px] sm:text-[52px] font-black tracking-[0.12em] text-neutral-900 leading-none">INUTILES</h3>
      <div className="w-20 h-[4px] bg-neutral-900 mt-6" />
      <p className="font-mono text-[13px] font-black tracking-[0.2em] text-neutral-800 uppercase mt-4">MANUAL DEL EMPLEADO</p>
      <p className="font-mono text-[11px] font-bold tracking-widest text-neutral-500 mt-2">ED. 2026 — BLANCO & NEGRO</p>
    </div>
    <p className="font-mono text-[11px] font-black tracking-widest text-neutral-400 text-center">— 01 —</p>
    <div className="absolute bottom-0 left-0 w-6 h-6 border-t-[2px] border-r-[2px] border-neutral-900/20 rounded-tr-xl" />
  </div>
);

// Portada - Derecha (indice)
const IndiceDer = ({ onJump }: { onJump: (id: string) => void }) => (
  <div className="flex-1 bg-white p-8 flex flex-col relative overflow-hidden">
    <div className="absolute top-8 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="absolute top-12 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="flex-1 flex flex-col justify-center gap-2">
      <p className="font-mono text-[11px] font-black tracking-[0.3em] text-neutral-900 border-b-2 border-neutral-900 pb-2">INDICE — 8 MODULOS</p>

      {[
        { n: "01", t: "COMO COBRAR", d: "Buscar · Carrito · F5", p: "03 — 04", id: "cobrar" },
        { n: "02", t: "INVENTARIO", d: "Buscar · Stock · Alertas", p: "05 — 06", id: "inventario" },
        { n: "03", t: "TICKETS Y CORTES", d: "Proximamente", p: "07 — 08", id: "tickets" },
        { n: "04", t: "CLIENTES", d: "Proximamente", p: "09 — 10", id: "clientes" },
        { n: "05", t: "MI PERFIL", d: "Turno · Salario · Metas", p: "11 — 12", id: "perfil" },
        { n: "06", t: "Y.A.R.V.I.S.", d: "Pregunta · Herramientas · Offline", p: "13 — 14", id: "yarvis" },
        { n: "07", t: "INUTIL", d: "Creador · Contacto · Repo", p: "15 — 16", id: "inutil" },
        { n: "08", t: "LICENCIA", d: "GPL V3 · Resumen", p: "17 — 18", id: "licencia" },
      ].map((item) => (
        <button
          key={item.id}
          onClick={(e) => {
            e.stopPropagation();
            onJump(item.id);
          }}
          className="flex items-center gap-3 text-left group/item hover:bg-neutral-50 rounded-xl p-1.5 -mx-2 transition-colors"
        >
          <span className="font-mono text-[10px] font-black text-neutral-900 border border-neutral-900 w-7 h-7 flex items-center justify-center rounded-lg group-hover/item:bg-neutral-900 group-hover/item:text-white transition-colors shrink-0">
            {item.n}
          </span>
          <div className="flex-1 min-w-0">
            <p className="font-mono text-[10px] font-black tracking-widest text-neutral-900 leading-none truncate">{item.t}</p>
            <p className="font-mono text-[8px] font-bold text-neutral-500 truncate">{item.d}</p>
          </div>
          <span className="font-mono text-[8px] font-black text-neutral-400 whitespace-nowrap">{item.p}</span>
        </button>
      ))}

      <div className="mt-1 border-2 border-neutral-900 rounded-2xl p-3 bg-neutral-50">
        <p className="font-mono text-[9px] font-black tracking-widest text-neutral-900 leading-relaxed">
          &gt; TOCA LA PAGINA PARA HOJEAR
          <br />
          &gt; O USA LOS SEPARADORES
          <span className="inline-block w-[9px] h-[11px] bg-neutral-900 ml-1.5 align-middle animate-pulse" />
        </p>
      </div>
    </div>
    <p className="font-mono text-[11px] font-black tracking-widest text-neutral-400 text-center">— 02 —</p>
    <div className="absolute bottom-0 right-0 w-6 h-6 border-t-[2px] border-l-[2px] border-neutral-900/20 rounded-tl-xl" />
  </div>
);

const Libro = () => {
  const [actual, setActual] = useState(0);
  const [siguiente, setSiguiente] = useState<number | null>(null);
  const [girando, setGirando] = useState(false);
  const [direccion, setDireccion] = useState<"next" | "prev">("next");

  const goTo = (id: string) => {
    const idx = spreads.findIndex((s) => s.id === id);
    if (idx === -1 || idx === actual || girando) return;
    setDireccion(idx > actual ? "next" : "prev");
    setSiguiente(idx);
    setGirando(true);
    setTimeout(() => {
      setActual(idx);
      setGirando(false);
      setSiguiente(null);
    }, 620);
  };

  const avanzar = () => {
    if (girando) return;
    const next = (actual + 1) % spreads.length;
    goTo(spreads[next].id);
  };

  const retroceder = () => {
    if (girando) return;
    const prev = (actual - 1 + spreads.length) % spreads.length;
    goTo(spreads[prev].id);
  };

  const handleBookClick = (e: React.MouseEvent<HTMLDivElement>) => {
    const rect = (e.currentTarget as HTMLDivElement).getBoundingClientRect();
    const x = e.clientX - rect.left;
    if (x < rect.width / 2) retroceder();
    else avanzar();
  };

  const spreads: Spread[] = [
    { id: "indice", left: <PortadaIzq />, right: <IndiceDer onJump={goTo} /> },
    { id: "cobrar", left: <CobrarIzq />, right: <CobrarDer /> },
    { id: "inventario", left: <InventarioIzq />, right: <InventarioDer /> },
    { id: "tickets", left: <TicketsIzq />, right: <TicketsDer /> },
    { id: "clientes", left: <ClientesIzq />, right: <ClientesDer /> },
    { id: "perfil", left: <PerfilIzq />, right: <PerfilDer /> },
    { id: "yarvis", left: <YarvisIzq />, right: <YarvisDer /> },
    { id: "inutil", left: <InutilIzq />, right: <InutilDer /> },
    { id: "licencia", left: <LicenciaIzq />, right: <LicenciaDer /> },
  ];

  const separadores = [
    cobrarSeparador,
    inventarioSeparador,
    ticketsSeparador,
    clientesSeparador,
    perfilSeparador,
    yarvisSeparador,
    inutilSeparador,
    licenciaSeparador,
  ];

  const spreadActual = spreads[actual];
  const spreadSiguiente = siguiente !== null ? spreads[siguiente] : null;

  return (
    <div className="w-full flex flex-col items-center mt-2">
      <style>{`
        .perspectiva { perspective: 2000px; }
        .preserve-3d { transform-style: preserve-3d; }
        .backface-hidden { backface-visibility: hidden; }
        .girando-next { animation: flipNext 620ms ease forwards; }
        .girando-prev { animation: flipPrev 620ms ease forwards; }
        @keyframes flipNext {
          0% { transform: rotateY(0deg); }
          100% { transform: rotateY(-180deg); }
        }
        @keyframes flipPrev {
          0% { transform: rotateY(-180deg); }
          100% { transform: rotateY(0deg); }
        }
      `}</style>

      <div
        onClick={handleBookClick}
        className="relative flex w-full max-w-[1200px] h-[620px] cursor-pointer select-none perspectiva group"
        title="click derecha avanza, izquierda retrocede, o usa separadores"
      >
        <div className="absolute inset-0 translate-y-3 bg-neutral-900/10 rounded-[2.2rem] blur-[1px]" />

        <div className="relative flex w-full h-full bg-white border-[3px] border-neutral-900 rounded-[1.8rem] overflow-hidden shadow-[0_12px_24px_rgba(0,0,0,0.12)]">
          <div className="flex-1 flex overflow-hidden">{spreadActual.left}</div>
          <div className="absolute left-1/2 top-0 bottom-0 w-[3px] bg-neutral-900 -translate-x-1/2 z-10" />
          <div className="absolute left-1/2 top-0 bottom-0 w-[14px] -translate-x-1/2 z-0 bg-gradient-to-r from-transparent via-neutral-900/10 to-transparent pointer-events-none" />
          <div className="flex-1 flex overflow-hidden">{spreadActual.right}</div>

          {girando && spreadSiguiente && (
            <div className="absolute top-0 bottom-0 right-0 w-1/2 z-30 preserve-3d origin-left">
              <div
                className={`absolute inset-0 preserve-3d w-full h-full ${direccion === "next" ? "girando-next" : "girando-prev"}`}
                style={{ transformOrigin: "left center" }}
              >
                <div className="absolute inset-0 backface-hidden overflow-hidden bg-white border-l-[3px] border-neutral-900">
                  <div className="w-[200%] h-full flex">
                    <div className="w-1/2 flex">{spreadActual.right}</div>
                    <div className="w-1/2 bg-white" />
                  </div>
                </div>
                <div
                  className="absolute inset-0 backface-hidden overflow-hidden bg-white border-l-[3px] border-neutral-900"
                  style={{ transform: "rotateY(180deg)" }}
                >
                  <div className="w-[200%] h-full flex flex-row-reverse">
                    <div className="w-1/2 flex">{spreadSiguiente.left}</div>
                    <div className="w-1/2 bg-white" />
                  </div>
                </div>
              </div>
              <div className="absolute inset-0 bg-gradient-to-r from-black/20 to-transparent pointer-events-none" />
            </div>
          )}
        </div>

        {separadores.map((s) => (
          <Separador
            key={s.id}
            label={s.label}
            icon={s.icon}
            left={s.left}
            activo={spreads[actual].id === s.id}
            onClick={() => goTo(s.id)}
          />
        ))}
      </div>

      <p className="font-mono text-[10px] font-bold tracking-[0.2em] text-neutral-400 uppercase mt-4 flex items-center gap-2">
        <span className="w-2 h-2 bg-neutral-900 rounded-full animate-pulse" />
        {spreads[actual].id.toUpperCase()} — {String(actual * 2 + 1).padStart(2, "0")} / {String(actual * 2 + 2).padStart(2, "0")}
        <span className="text-neutral-300">·</span> CLICK PAGINA PARA HOJEAR · SEPARADORES PARA SALTAR
      </p>
    </div>
  );
};

export default Libro;
