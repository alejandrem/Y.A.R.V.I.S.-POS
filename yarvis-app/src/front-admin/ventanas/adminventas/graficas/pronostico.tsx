// Gráfica corazón: histórico real + pronóstico Holt-Winters con banda 95%.
// Un solo comando trae ambas series para que la curva sea continua.
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  ComposedChart, Line, Area, XAxis, YAxis, CartesianGrid,
  Tooltip, ResponsiveContainer, ReferenceLine,
} from "recharts";
import { RangoDropdown, moneda, fechaCorta, Tarjeta, TituloSeccion, Vacio, Cargando } from "./controles";

interface PuntoH {
  fecha: string;
  total: number;
}
interface PuntoP {
  fecha: string;
  prediccion: number;
  minimo: number;
  maximo: number;
}

const HORIZONTES = [
  { valor: "7", etiqueta: "7 días", dias: 7 },
  { valor: "15", etiqueta: "15 días", dias: 15 },
  { valor: "30", etiqueta: "30 días", dias: 30 },
  { valor: "90", etiqueta: "3 meses", dias: 90 },
  { valor: "custom", etiqueta: "Personalizado…" },
];

const HISTORIA_VISIBLE = 60;

const Pronostico = () => {
  const [horizonte, setHorizonte] = useState("30");
  const [customDias, setCustomDias] = useState<number | undefined>(undefined);
  const [historial, setHistorial] = useState<PuntoH[]>([]);
  const [pronostico, setPronostico] = useState<PuntoP[]>([]);
  const [estado, setEstado] = useState<"cargando" | "ok" | "vacio" | "error">("cargando");
  const [mensaje, setMensaje] = useState("");

  useEffect(() => {
    let vivo = true;
    const dias = horizonte === "custom" ? customDias ?? 30 : Number(horizonte);
    invoke<{ historial: PuntoH[]; pronostico: PuntoP[] }>("get_ventas_con_pronostico", { days: dias })
      .then((r) => {
        if (!vivo) return;
        setHistorial(r.historial);
        setPronostico(r.pronostico);
        setEstado("ok");
      })
      .catch((e) => {
        if (!vivo) return;
        const msg = String(e);
        setMensaje(msg);
        setEstado(msg.includes("Datos insuficientes") || msg.includes("No hay ventas") ? "vacio" : "error");
      });
    return () => {
      vivo = false;
    };
  }, [horizonte, customDias]);

  const hist = historial.slice(-HISTORIA_VISIBLE);
  const datos = [
    ...hist.map((h) => ({ fecha: h.fecha, real: h.total })),
    ...pronostico.map((p) => ({ fecha: p.fecha, pron: p.prediccion, banda: [p.minimo, p.maximo] })),
  ];
  const corte = hist.length ? hist[hist.length - 1].fecha : "";

  return (
    <Tarjeta>
      <TituloSeccion
        titulo="Pronóstico"
        control={
          <div className="flex items-center gap-2">
            <span className="text-[10px] font-black uppercase tracking-widest text-neutral-400">
              <span className="text-emerald-500">●</span> modelo activo
            </span>
            <RangoDropdown
              valor={horizonte}
              opciones={HORIZONTES}
              onChange={(v, c) => {
                setHorizonte(v);
                setCustomDias(c);
              }}
            />
          </div>
        }
      />
      {estado === "cargando" && <Cargando />}
      {estado === "vacio" && (
        <Vacio mensaje={mensaje || "Se necesitan al menos 4 días con ventas para pronosticar."} />
      )}
      {estado === "error" && <Vacio mensaje={`No se pudo calcular: ${mensaje}`} />}
      {estado === "ok" && (
        <>
          <div className="h-72 sm:h-80">
            <ResponsiveContainer width="100%" height="100%">
              <ComposedChart data={datos} margin={{ top: 5, right: 10, left: 0, bottom: 0 }}>
                <CartesianGrid stroke="#f0f0f0" vertical={false} />
                <XAxis
                  dataKey="fecha"
                  tickFormatter={fechaCorta}
                  tick={{ fontSize: 10, fill: "#a3a3a3" }}
                  minTickGap={40}
                />
                <YAxis
                  tickFormatter={(v: number) => `$${v >= 1000 ? `${(v / 1000).toFixed(0)}k` : v}`}
                  tick={{ fontSize: 10, fill: "#a3a3a3" }}
                  width={45}
                />
                <Tooltip
                  formatter={(v: unknown, name: unknown) =>
                    [moneda(Number(v ?? 0)), name === "real" ? "Real" : name === "pron" ? "Pronóstico" : "Banda"]
                  }
                  labelFormatter={(l: unknown) => String(l)}
                />
                {corte && <ReferenceLine x={corte} stroke="#171717" strokeDasharray="4 4" label={{ value: "hoy", fontSize: 10, fill: "#737373" }} />}
                <Area type="monotone" dataKey="banda" stroke="none" fill="#e5e5e5" name="banda" connectNulls />
                <Line type="monotone" dataKey="real" stroke="#171717" strokeWidth={2.5} dot={false} name="real" connectNulls />
                <Line type="monotone" dataKey="pron" stroke="#737373" strokeWidth={2.5} strokeDasharray="6 4" dot={false} name="pron" connectNulls />
              </ComposedChart>
            </ResponsiveContainer>
          </div>
          <p className="text-[11px] text-neutral-400 font-bold mt-3">
            ─── real &nbsp; ─╌ pronóstico &nbsp; <span className="bg-neutral-200 px-1 rounded">banda 95%</span> → ventas irregulares: la banda se abre con el horizonte
          </p>
        </>
      )}
    </Tarjeta>
  );
};

export default Pronostico;
