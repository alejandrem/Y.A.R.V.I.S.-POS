import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { 
  GastoRecurrente, CrearGastoRequest, PagoGasto, RegistrarPagoRequest,
  CorteCaja, FiltrosCortes, MovimientoCaja, CorteDetalle, CrearCorteRequest, CerrarCorteRequest,
  MetricasUtilidad, ResumenPeriodo, DatoGraficaPL, DatoGraficaGastosCategoria, DatoGraficaCortesZ,
  PuntoEquilibrio, AlertaFinanciera, FiltrosPeriodo
} from './types';

export function useGastos() {
  const [gastos, setGastos] = useState<GastoRecurrente[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const cargar = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await invoke<GastoRecurrente[]>('get_gastos_recurrentes');
      setGastos(data);
    } catch (e) {
      setError(e as string);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { cargar(); }, [cargar]);

  const crear = async (gasto: CrearGastoRequest) => {
    const id = await invoke<number>('crear_gasto', { gasto });
    await cargar();
    return id;
  };

  const actualizar = async (id: number, gasto: CrearGastoRequest) => {
    await invoke('actualizar_gasto', { id, gasto });
    await cargar();
  };

  const eliminar = async (id: number) => {
    await invoke('eliminar_gasto', { id });
    await cargar();
  };

  const registrarPago = async (pago: RegistrarPagoRequest) => {
    const id = await invoke<number>('registrar_pago_gasto', { pago });
    await cargar();
    return id;
  };

  const getPagos = async (gasto_id: number) => {
    return await invoke<PagoGasto[]>('get_pagos_gasto', { gasto_id });
  };

  const getProximosVencimientos = async (dias: number = 30) => {
    return await invoke<GastoRecurrente[]>('get_proximos_vencimientos', { dias });
  };

  return { gastos, loading, error, cargar, crear, actualizar, eliminar, registrarPago, getPagos, getProximosVencimientos };
}

export function useCortes() {
  const [cortes, setCortes] = useState<CorteCaja[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const cargar = useCallback(async (filtros: FiltrosCortes = {}) => {
    setLoading(true);
    setError(null);
    try {
      const data = await invoke<CorteCaja[]>('get_cortes_caja', { filtros });
      setCortes(data);
    } catch (e) {
      setError(e as string);
    } finally {
      setLoading(false);
    }
  }, []);

  const getDetalle = async (corte_id: number) => {
    return await invoke<CorteDetalle>('get_corte_detalle', { corte_id });
  };

  const crearCorteX = async (datos: CrearCorteRequest) => {
    const id = await invoke<number>('crear_corte_x', { datos });
    await cargar();
    return id;
  };

  const crearCorteZ = async (datos: CrearCorteRequest) => {
    const id = await invoke<number>('crear_corte_z', { datos });
    await cargar();
    return id;
  };

  const cerrarCorte = async (datos: CerrarCorteRequest) => {
    await invoke('cerrar_corte', datos);
    await cargar();
  };

  const agregarMovimiento = async (mov: MovimientoCaja) => {
    const id = await invoke<number>('agregar_movimiento_caja', { mov });
    return id;
  };

  const getMovimientos = async (corte_id: number) => {
    return await invoke<MovimientoCaja[]>('get_movimientos_corte', { corte_id });
  };

  return { cortes, loading, error, cargar, getDetalle, crearCorteX, crearCorteZ, cerrarCorte, agregarMovimiento, getMovimientos };
}

export function useMetricas() {
  const [resumen, setResumen] = useState<ResumenPeriodo | null>(null);
  const [metricasDiarias, setMetricasDiarias] = useState<MetricasUtilidad[]>([]);
  const [puntoEquilibrio, setPuntoEquilibrio] = useState<PuntoEquilibrio | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const cargarResumen = useCallback(async (fecha_inicio: string, fecha_fin: string) => {
    setLoading(true);
    setError(null);
    try {
      const [res, met, pe] = await Promise.all([
        invoke<ResumenPeriodo>('get_resumen_periodo', { fecha_inicio, fecha_fin }),
        invoke<MetricasUtilidad[]>('get_metricas_diarias', { fecha_inicio, fecha_fin }),
        invoke<PuntoEquilibrio>('get_punto_equilibrio'),
      ]);
      setResumen(res);
      setMetricasDiarias(met);
      setPuntoEquilibrio(pe);
    } catch (e) {
      setError(e as string);
    } finally {
      setLoading(false);
    }
  }, []);

  const recalcularDia = async (fecha: string) => {
    await invoke('recalcular_resumen_diario', { fecha });
  };

  return { resumen, metricasDiarias, puntoEquilibrio, loading, error, cargarResumen, recalcularDia };
}

export function useGraficas() {
  const [datosPL, setDatosPL] = useState<DatoGraficaPL[]>([]);
  const [gastosCategoria, setGastosCategoria] = useState<DatoGraficaGastosCategoria[]>([]);
  const [tendenciaCortesZ, setTendenciaCortesZ] = useState<DatoGraficaCortesZ[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const cargarPL = useCallback(async (fecha_inicio: string, fecha_fin: string, granularidad: 'dia' | 'semana' | 'mes' = 'mes') => {
    setLoading(true);
    setError(null);
    try {
      const data = await invoke<DatoGraficaPL[]>('get_datos_grafica_pl', { fecha_inicio, fecha_fin, granularidad });
      setDatosPL(data);
    } catch (e) {
      setError(e as string);
    } finally {
      setLoading(false);
    }
  }, []);

  const cargarGastosCategoria = useCallback(async (fecha_inicio: string, fecha_fin: string) => {
    try {
      const data = await invoke<DatoGraficaGastosCategoria[]>('get_gastos_por_categoria', { fecha_inicio, fecha_fin });
      setGastosCategoria(data);
    } catch (e) {
      setError(e as string);
    }
  }, []);

  const cargarTendenciaCortesZ = useCallback(async (fecha_inicio: string, fecha_fin: string) => {
    try {
      const data = await invoke<DatoGraficaCortesZ[]>('get_tendencia_cortes_z', { fecha_inicio, fecha_fin });
      setTendenciaCortesZ(data);
    } catch (e) {
      setError(e as string);
    }
  }, []);

  const cargarTodo = async (fecha_inicio: string, fecha_fin: string) => {
    await Promise.all([
      cargarPL(fecha_inicio, fecha_fin),
      cargarGastosCategoria(fecha_inicio, fecha_fin),
      cargarTendenciaCortesZ(fecha_inicio, fecha_fin),
    ]);
  };

  return { datosPL, gastosCategoria, tendenciaCortesZ, loading, error, cargarPL, cargarGastosCategoria, cargarTendenciaCortesZ, cargarTodo };
}

export function useAlertas() {
  const [alertas, setAlertas] = useState<AlertaFinanciera[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const cargar = useCallback(async (soloNoLeidas = false) => {
    setLoading(true);
    setError(null);
    try {
      const data = await invoke<AlertaFinanciera[]>('get_alertas', { solo_no_leidas: soloNoLeidas });
      setAlertas(data);
    } catch (e) {
      setError(e as string);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { cargar(); }, [cargar]);

  const marcarLeida = async (id: number) => {
    await invoke('marcar_alerta_leida', { id });
    setAlertas(prev => prev.map(a => a.id === id ? { ...a, leida: true } : a));
  };

  const marcarTodasLeidas = async () => {
    await invoke('marcar_todas_alertas_leidas');
    setAlertas(prev => prev.map(a => ({ ...a, leida: true })));
  };

  const generarAutomaticas = async () => {
    const nuevas = await invoke<AlertaFinanciera[]>('generar_alertas_automaticas');
    setAlertas(prev => [...nuevas, ...prev]);
    return nuevas;
  };

  return { alertas, loading, error, cargar, marcarLeida, marcarTodasLeidas, generarAutomaticas };
}