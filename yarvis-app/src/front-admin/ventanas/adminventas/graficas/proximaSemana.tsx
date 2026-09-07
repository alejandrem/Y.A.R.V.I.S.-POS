// Próxima semana: 3 frases con reglas fijas sobre los números del
// pronóstico. NADA de LLM: determinista, cero RAM extra.
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { moneda, Tarjeta, TituloSeccion, Vacio, Cargando } from "./controles";

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

const DIAS = ["domingo", "lunes", "martes", "miércoles", "jueves", "viernes", "sábado"];

const diaSemana = (iso: string) => {
  const [y, m, d] = iso.split("-").map(Number);
  return DIAS[new Date(y, m - 1, d).getDay()];
};

/** Traduce los números del pronóstico a 3 frases con reglas fijas.
/// Sin LLM: determinista, cero RAM. Puro y testeable. */
export const redactarSugerencias = (
  historial: PuntoH[],
  pronostico: PuntoP[],
): { resumen: string; lineas: string[] } | null => {
  if (!pronostico.length) return null;
  const total = pronostico.reduce((a, p) => a + p.prediccion, 0);
  const min = pronostico.reduce((a, p) => a + p.minimo, 0);
  const max = pronostico.reduce((a, p) => a + p.maximo, 0);
  const mejor = pronostico.reduce((a, b) => (b.prediccion > a.prediccion ? b : a));
  const peor = pronostico.reduce((a, b) => (b.prediccion < a.prediccion ? b : a));
  const ultimos7 = historial.slice(-7).reduce((a, h) => a + h.total, 0);
  const lineas = [
    `Tu mejor día será el ${diaSemana(mejor.fecha)} (~${moneda(mejor.prediccion)}): surte refresco y botana un día antes.`,
    `El ${diaSemana(peor.fecha)} pinta flojo (~${moneda(peor.prediccion)}): úsalo para inventario y pedidos a proveedor.`,
  ];
  if (ultimos7 > 0) {
    const pct = ((total - ultimos7) / ultimos7) * 100;
    lineas.push(
      pct >= 0
        ? `Vas ${pct.toFixed(0)}% arriba vs los últimos 7 días a estas alturas.`
        : `Vas ${Math.abs(pct).toFixed(0)}% abajo vs los últimos 7 días: semana de cuidar el gasto.`,
    );
  }
  return {
    resumen: `Se esperan ${moneda(total)} en total (entre ${moneda(min)} y ${moneda(max)}).`,
    lineas,
  };
};

const ProximaSemana = () => {
  const [texto, setTexto] = useState<string[] | null>(null);
  const [resumen, setResumen] = useState("");
  const [sinDatos, setSinDatos] = useState(false);

  useEffect(() => {
    let vivo = true;
    invoke<{ historial: PuntoH[]; pronostico: PuntoP[] }>("get_ventas_con_pronostico", { days: 7 })
      .then(({ historial, pronostico }) => {
        if (!vivo) return;
        const r = redactarSugerencias(historial, pronostico);
        if (!r) {
          setSinDatos(true);
          return;
        }
        setTexto(r.lineas);
        setResumen(r.resumen);
      })
      .catch(() => {
        if (vivo) setSinDatos(true);
      });
    return () => {
      vivo = false;
    };
  }, []);

  return (
    <Tarjeta>
      <TituloSeccion titulo="Próxima semana · lo que calcula el modelo" />
      {texto === null && !sinDatos && <Cargando />}
      {sinDatos && (
        <Vacio mensaje="Sin historial suficiente para sugerir (mínimo 4 días con ventas)." />
      )}
      {texto !== null && texto.length > 0 && (
        <>
          <p className="text-sm font-black text-neutral-900">{resumen}</p>
          <ul className="mt-3 space-y-2">
            {texto.map((l, i) => (
              <li key={i} className="flex gap-2.5 text-sm text-neutral-600 font-bold">
                <span className="text-neutral-900">●</span>
                {l}
              </li>
            ))}
          </ul>
          <p className="text-[11px] text-neutral-400 font-bold mt-4">
            Números de Holt-Winters con tu historial, sin adivinanzas.
          </p>
        </>
      )}
    </Tarjeta>
  );
};

export default ProximaSemana;
