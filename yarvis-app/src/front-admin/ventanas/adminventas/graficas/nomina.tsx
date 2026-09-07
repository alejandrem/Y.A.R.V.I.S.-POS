// Costo de nómina: barras por empleado (semanal), por día o total semanal,
// contra la venta semanal. Costo programado según turnos activos.
import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, ReferenceLine } from "recharts";
import { RangoDropdown, RANGOS_NOMINA, moneda, Tarjeta, TituloSeccion, Vacio, Cargando } from "./controles";

interface Empleado {
  id: number;
  nombre: string;
  estado: string;
  salario_semanal: number;
  salario_diario: number;
}

interface Resumen {
  total_ventas: number;
}

const isoHace = (dias: number) => {
  const d = new Date();
  d.setDate(d.getDate() - dias);
  return d.toISOString().slice(0, 10);
};

/** Costo programado por empleado activo (puro, testeable). */
export const resumirNomina = (empleados: Empleado[]) => {
  const activos = empleados.filter((e) => e.estado === "activo");
  const filas = activos.map((e) => ({
    nombre: e.nombre.split(" ")[0],
    total: e.salario_semanal || e.salario_diario * 7,
  }));
  const totalSemanal = filas.reduce((a, f) => a + f.total, 0);
  const totalDiario = activos.reduce((a, e) => a + (e.salario_diario || e.salario_semanal / 7), 0);
  return { filas, totalSemanal, totalDiario };
};

const Nomina = () => {
  const [modo, setModo] = useState("empleado");
  const [empleados, setEmpleados] = useState<Empleado[] | null>(null);
  const [ventaSemanal, setVentaSemanal] = useState<number | null>(null);

  useEffect(() => {
    let vivo = true;
    Promise.all([
      invoke<Empleado[]>("get_empleados").catch(() => [] as Empleado[]),
      invoke<Resumen>("get_resumen_periodo", { fechaInicio: isoHace(6), fechaFin: isoHace(0) }).catch(
        () => ({ total_ventas: 0 } as Resumen),
      ),
    ]).then(([e, r]) => {
      if (!vivo) return;
      setEmpleados(e.filter((x) => x.estado === "activo"));
      setVentaSemanal(r.total_ventas);
    });
    return () => {
      vivo = false;
    };
  }, []);

  const datos = useMemo(() => {
    if (!empleados) return null;
    const { filas, totalSemanal, totalDiario } = resumirNomina(empleados);
    if (modo === "empleado") return filas;
    if (modo === "dia") {
      const dias = ["Lun", "Mar", "Mié", "Jue", "Vie", "Sáb", "Dom"];
      return dias.map((nombre) => ({ nombre, total: totalDiario }));
    }
    return [{ nombre: "Semana", total: totalSemanal }];
  }, [empleados, modo]);

  const totalSemanal = useMemo(
    () => (empleados ? resumirNomina(empleados).totalSemanal : 0),
    [empleados],
  );
  const ratio = ventaSemanal ? (totalSemanal / ventaSemanal) * 100 : 0;

  return (
    <Tarjeta>
      <TituloSeccion
        titulo="Costo de nómina"
        control={
          <div className="flex items-center gap-2">
            <span className="text-[10px] font-black uppercase tracking-widest text-neutral-400">
              Total semanal: {moneda(totalSemanal)}
            </span>
            <RangoDropdown valor={modo} opciones={RANGOS_NOMINA} onChange={(v) => setModo(v)} conCustom={false} />
          </div>
        }
      />
      {!datos && <Cargando />}
      {datos && datos.length === 0 && <Vacio mensaje="Sin empleados activos." />}
      {datos && datos.length > 0 && (
        <>
          <div className="h-64">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={datos} margin={{ top: 5, right: 10, left: 0, bottom: 0 }}>
                <CartesianGrid stroke="#f0f0f0" vertical={false} />
                <XAxis dataKey="nombre" tick={{ fontSize: 11, fill: "#404040", fontWeight: 700 }} interval={0} angle={modo === "dia" ? 0 : -15} dy={8} height={50} />
                <YAxis
                  tickFormatter={(v: number) => `$${v >= 1000 ? `${(v / 1000).toFixed(0)}k` : v}`}
                  tick={{ fontSize: 10, fill: "#a3a3a3" }}
                  width={45}
                />
                <Tooltip formatter={(v: unknown) => [moneda(Number(v ?? 0)), "Costo"]} />
                {modo === "empleado" && ventaSemanal ? (
                  <ReferenceLine
                    y={ventaSemanal * 0.35}
                    stroke="#ef4444"
                    strokeDasharray="5 4"
                    label={{ value: "35% de la venta", fontSize: 10, fill: "#ef4444" }}
                  />
                ) : null}
                <Bar dataKey="total" fill="#171717" radius={[8, 8, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </div>
          <p className="text-[11px] text-neutral-400 font-bold mt-3">
            La nómina semanal equivale al {ratio.toFixed(0)}% de tu venta semanal
            {ratio > 35 ? ". Ojo si pasa del 35%." : "."} Costo programado según turnos activos.
          </p>
        </>
      )}
    </Tarjeta>
  );
};

export default Nomina;
