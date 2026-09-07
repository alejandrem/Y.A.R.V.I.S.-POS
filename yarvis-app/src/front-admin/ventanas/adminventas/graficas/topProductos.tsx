// Top 5 productos por ingreso en el rango elegido (detalle real SQL).
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Cell,
} from "recharts";
import { RangoDropdown, RANGOS_VENTANA, moneda, Tarjeta, TituloSeccion, Vacio, Cargando } from "./controles";

interface TopProducto {
  nombre: string;
  total: number;
  cantidad: number;
}

const COLORES = ["#171717", "#404040", "#737373", "#a3a3a3", "#d4d4d4"];

const TopProductos = () => {
  const [rango, setRango] = useState("30");
  const [customDias, setCustomDias] = useState<number | undefined>(undefined);
  const [datos, setDatos] = useState<TopProducto[] | null>(null);

  useEffect(() => {
    let vivo = true;
    const dias = rango === "custom" ? customDias ?? 30 : Number(rango);
    setDatos(null);
    invoke<TopProducto[]>("get_top_productos", { days: dias })
      .then((r) => vivo && setDatos(r))
      .catch(() => vivo && setDatos([]));
    return () => {
      vivo = false;
    };
  }, [rango, customDias]);

  return (
    <Tarjeta>
      <TituloSeccion
        titulo="Top 5 productos"
        control={
          <RangoDropdown
            valor={rango}
            opciones={RANGOS_VENTANA}
            onChange={(v, c) => {
              setRango(v);
              setCustomDias(c);
            }}
          />
        }
      />
      {datos === null && <Cargando />}
      {datos !== null && datos.length === 0 && <Vacio mensaje="Sin ventas en este rango." />}
      {datos !== null && datos.length > 0 && (
        <div className="h-64">
          <ResponsiveContainer width="100%" height="100%">
            <BarChart data={datos} layout="vertical" margin={{ top: 0, right: 10, left: 0, bottom: 0 }}>
              <CartesianGrid stroke="#f0f0f0" horizontal={false} />
              <XAxis type="number" tickFormatter={(v: number) => `$${v >= 1000 ? `${(v / 1000).toFixed(0)}k` : v}`} tick={{ fontSize: 10, fill: "#a3a3a3" }} />
              <YAxis type="category" dataKey="nombre" tick={{ fontSize: 11, fill: "#404040", fontWeight: 700 }} width={130} />
              <Tooltip formatter={(v: unknown) => [moneda(Number(v ?? 0)), "Ingreso"]} />
              <Bar dataKey="total" radius={[0, 8, 8, 0]}>
                {datos.map((_, i) => (
                  <Cell key={i} fill={COLORES[i % COLORES.length]} />
                ))}
              </Bar>
            </BarChart>
          </ResponsiveContainer>
        </div>
      )}
    </Tarjeta>
  );
};

export default TopProductos;
