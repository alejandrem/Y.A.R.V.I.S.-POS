import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { 
  ResumenPeriodo, PuntoEquilibrio, DatoGraficaPL, AlertaFinanciera 
} from './types';

export function useMetricas() {
  const [resumen, setResumen] = useState<ResumenPeriodo | null>(null);
  const [puntoEquilibrio, setPuntoEquilibrio] = useState<PuntoEquilibrio | null>(null);
  const [loading, setLoading] = useState(false);

  const cargarResumen = useCallback(async (fecha_inicio: string, fecha_fin: string) => {
    setLoading(true);
    try {
      const [res, pe] = await Promise.all([
        invoke<ResumenPeriodo>('get_resumen_periodo', { fecha_inicio, fecha_fin }),
        invoke<PuntoEquilibrio>('get_punto_equilibrio'),
      ]);
      setResumen(res);
      setPuntoEquilibrio(pe);
    } catch (error) {
      console.error('Error cargando métricas:', error);
    } finally {
      setLoading(false);
    }
  }, []);

  return { resumen, puntoEquilibrio, loading, cargarResumen };
}

export function useGraficas() {
  const [datosPL, setDatosPL] = useState<DatoGraficaPL[]>([]);
  const [loading, setLoading] = useState(false);

  const cargarPL = useCallback(async (fecha_inicio: string, fecha_fin: string, granularidad: 'dia' | 'semana' | 'mes') => {
    setLoading(true);
    try {
      const data = await invoke<DatoGraficaPL[]>('get_datos_grafica_pl', { fecha_inicio, fecha_fin, granularidad });
      setDatosPL(data);
    } catch (error) {
      console.error('Error cargando gráficas:', error);
    } finally {
      setLoading(false);
    }
  }, []);

  return { datosPL, loading, cargarPL };
}

export function useGastos() {
  const [gastos, setGastos] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);

  const cargarGastos = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invoke<any[]>('get_gastos_recurrentes');
      setGastos(data);
    } catch (error) {
      console.error('Error cargando gastos:', error);
    } finally {
      setLoading(false);
    }
  }, []);

  const crearGasto = useCallback(async (gasto: any) => {
    const id = await invoke<number>('crear_gasto', { gasto });
    await cargarGastos();
    return id;
  }, [cargarGastos]);

  const actualizarGasto = useCallback(async (id: number, gasto: any) => {
    await invoke('actualizar_gasto', { id, gasto });
    await cargarGastos();
  }, [cargarGastos]);

  const eliminarGasto = useCallback(async (id: number) => {
    await invoke('eliminar_gasto', { id });
    await cargarGastos();
  }, [cargarGastos]);

  const registrarPago = useCallback(async (pago: any) => {
    await invoke('registrar_pago_gasto', { pago });
    await cargarGastos();
  }, [cargarGastos]);

  return { gastos, loading, cargarGastos, crearGasto, actualizarGasto, eliminarGasto, registrarPago };
}

export function useCortes() {
  const [cortes, setCortes] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);

  const cargarCortes = useCallback(async (filtros: any = {}) => {
    setLoading(true);
    try {
      const data = await invoke<any[]>('get_cortes_caja', { filtros });
      setCortes(data);
    } catch (error) {
      console.error('Error cargando cortes:', error);
    } finally {
      setLoading(false);
    }
  }, []);

  const crearCorteX = useCallback(async (datos: any) => {
    const id = await invoke<number>('crear_corte_x', { datos });
    await cargarCortes();
    return id;
  }, [cargarCortes]);

  const crearCorteZ = useCallback(async (datos: any) => {
    const id = await invoke<number>('crear_corte_z', { datos });
    await cargarCortes();
    return id;
  }, [cargarCortes]);

  const cerrarCorte = useCallback(async (corte_id: number, datos: any) => {
    await invoke('cerrar_corte', { corte_id, ...datos });
    await cargarCortes();
  }, [cargarCortes]);

  const agregarMovimiento = useCallback(async (mov: any) => {
    await invoke('agregar_movimiento_caja', { mov });
  }, []);

  return { cortes, loading, cargarCortes, crearCorteX, crearCorteZ, cerrarCorte, agregarMovimiento };
}

export function useAlertas() {
  const [alertas, setAlertas] = useState<AlertaFinanciera[]>([]);
  const [loading, setLoading] = useState(false);

  const cargarAlertas = useCallback(async (solo_no_leidas = false) => {
    setLoading(true);
    try {
      const data = await invoke<AlertaFinanciera[]>('get_alertas', { solo_no_leidas });
      setAlertas(data);
    } catch (error) {
      console.error('Error cargando alertas:', error);
    } finally {
      setLoading(false);
    }
  }, []);

  const marcarLeida = useCallback(async (id: number) => {
    await invoke('marcar_alerta_leida', { id });
    setAlertas(prev => prev.map(a => a.id === id ? { ...a, leida: true } : a));
  }, []);

  const generarAlertas = useCallback(async () => {
    const nuevas = await invoke<AlertaFinanciera[]>('generar_alertas_automaticas');
    setAlertas(prev => [...nuevas, ...prev]);
    return nuevas;
  }, []);

  return { alertas, loading, cargarAlertas, marcarLeida, generarAlertas };
}

export function useExport() {
  const exportarGastosCSV = useCallback(async (fecha_inicio: string, fecha_fin: string) => {
    return await invoke<string>('exportar_gastos_csv', { fecha_inicio, fecha_fin });
  }, []);

  const exportarBalancePDF = useCallback(async (fecha_inicio: string, fecha_fin: string) => {
    return await invoke<Uint8Array>('exportar_balance_pdf', { fecha_inicio, fecha_fin });
  }, []);

  return { exportarGastosCSV, exportarBalancePDF };
}