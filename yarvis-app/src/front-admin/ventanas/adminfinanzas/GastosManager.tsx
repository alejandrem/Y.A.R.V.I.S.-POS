import { useState, useEffect, useMemo } from 'react';
import { useGastos } from './hooks';
import {
  GastoRecurrente, CrearGastoRequest, RegistrarPagoRequest,
  TipoGasto, CategoriaGasto, FrecuenciaGasto, EstadoPago,
  TIPOS_GASTO, CATEGORIAS_GASTO, FRECUENCIAS_GASTO, DIAS_SEMANA,
  COLORES_ESTADO_PAGO,
} from './types';
import { formatMXN, getFechaHoy } from './utils';

const EMPTY_FORM: CrearGastoRequest = {
  nombre: '',
  tipo: 'varios',
  categoria: 'varios',
  monto_proyectado: 0,
  frecuencia: 'mensual',
  fecha_inicio: getFechaHoy(),
};

const EMPTY_PAGO = {
  fecha_pago: getFechaHoy(),
  monto_pagado: 0,
  metodo_pago: 'efectivo',
  folio_comprobante: '',
  notas: '',
};

const GastosManager = () => {
  const { gastos, loading, cargarGastos, crearGasto, actualizarGasto, eliminarGasto, registrarPago } = useGastos();
  const [filtroEstado, setFiltroEstado] = useState<EstadoPago | 'todos'>('todos');
  const [filtroTipo, setFiltroTipo] = useState<TipoGasto | 'todos'>('todos');
  const [modalGasto, setModalGasto] = useState(false);
  const [modalPago, setModalPago] = useState(false);
  const [editando, setEditando] = useState<GastoRecurrente | null>(null);
  const [gastoParaPago, setGastoParaPago] = useState<GastoRecurrente | null>(null);
  const [form, setForm] = useState<CrearGastoRequest>(EMPTY_FORM);
  const [formPago, setFormPago] = useState(EMPTY_PAGO);
  const [guardando, setGuardando] = useState(false);
  const [successMsg, setSuccessMsg] = useState('');

  useEffect(() => { cargarGastos(); }, [cargarGastos]);

  useEffect(() => {
    if (editando) {
      setForm({
        nombre: editando.nombre,
        tipo: editando.tipo as TipoGasto,
        categoria: editando.categoria as CategoriaGasto,
        monto_proyectado: editando.monto_proyectado,
        frecuencia: editando.frecuencia as FrecuenciaGasto,
        dia_pago: editando.dia_pago ?? undefined,
        intervalo_dias: editando.intervalo_dias ?? undefined,
        fecha_inicio: editando.fecha_inicio,
        fecha_fin: editando.fecha_fin ?? undefined,
        folio_comprobante: editando.folio_comprobante ?? undefined,
        notas: editando.notas ?? undefined,
      });
    } else {
      setForm(EMPTY_FORM);
    }
  }, [editando]);

  const gastosFiltrados = useMemo(() => {
    return (gastos as GastoRecurrente[]).filter(g => {
      if (filtroEstado !== 'todos' && g.estado_pago !== filtroEstado) return false;
      if (filtroTipo !== 'todos' && g.tipo !== filtroTipo) return false;
      return true;
    });
  }, [gastos, filtroEstado, filtroTipo]);

  const totalProyectado = useMemo(() =>
    gastosFiltrados.reduce((a, g) => a + g.monto_proyectado, 0), [gastosFiltrados]);

  const mostrarDiaMes = ['mensual', 'trimestral'].includes(form.frecuencia);
  const mostrarDiaSemana = form.frecuencia === 'semanal';
  const mostrarIntervalo = form.frecuencia === 'personalizado';

  const toast = (msg: string) => {
    setSuccessMsg(msg);
    setTimeout(() => setSuccessMsg(''), 3000);
  };

  const handleGuardar = async () => {
    if (!form.nombre.trim() || !form.monto_proyectado) return;
    setGuardando(true);
    try {
      if (editando) {
        await actualizarGasto(editando.id, form);
        toast('Gasto actualizado correctamente');
      } else {
        await crearGasto(form);
        toast('Gasto creado correctamente');
      }
      setModalGasto(false);
      setEditando(null);
    } catch (e) {
      console.error(e);
    } finally {
      setGuardando(false);
    }
  };

  const handleRegistrarPago = async () => {
    if (!gastoParaPago || !formPago.monto_pagado) return;
    setGuardando(true);
    try {
      const pago: RegistrarPagoRequest = {
        gasto_id: gastoParaPago.id,
        fecha_pago: formPago.fecha_pago,
        monto_pagado: formPago.monto_pagado,
        metodo_pago: formPago.metodo_pago || undefined,
        folio_comprobante: formPago.folio_comprobante || undefined,
        notas: formPago.notas || undefined,
      };
      await registrarPago(pago);
      setModalPago(false);
      setGastoParaPago(null);
      setFormPago(EMPTY_PAGO);
      toast('Pago registrado correctamente');
    } catch (e) {
      console.error(e);
    } finally {
      setGuardando(false);
    }
  };

  const handleEliminar = async (gasto: GastoRecurrente) => {
    if (!confirm(`¿Eliminar el gasto "${gasto.nombre}"? Esta acción no se puede deshacer.`)) return;
    await eliminarGasto(gasto.id);
    toast('Gasto eliminado');
  };

  const abrirPago = (gasto: GastoRecurrente) => {
    setGastoParaPago(gasto);
    setFormPago({ ...EMPTY_PAGO, monto_pagado: gasto.monto_proyectado });
    setModalPago(true);
  };

  return (
    <div className="space-y-8 animate-in fade-in duration-500">

      <div className="relative flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h3 className="text-xl font-black text-neutral-900 uppercase tracking-tight">Gastos Recurrentes</h3>
          <p className="text-[10px] text-neutral-400 font-black uppercase tracking-widest mt-0.5">Gestión de egresos operativos</p>
        </div>
        {successMsg && (
          <div className="absolute top-0 right-36 sm:right-48 bg-neutral-100 border border-neutral-300 text-neutral-800 px-4 py-2 rounded-2xl text-[10px] font-black uppercase tracking-widest animate-in fade-in slide-in-from-top-2 duration-300 shadow">
            {successMsg}
          </div>
        )}
        <button
          onClick={() => { setEditando(null); setModalGasto(true); }}
          className="px-5 py-3 bg-neutral-900 text-white text-[10px] font-black uppercase tracking-widest rounded-2xl hover:scale-[1.02] active:scale-95 transition-all shadow-lg shrink-0"
        >
          + Nuevo Gasto
        </button>
      </div>

      <div className="flex flex-wrap gap-3 items-center">
        <select
          value={filtroEstado}
          onChange={e => setFiltroEstado(e.target.value as any)}
          className="px-4 py-2 bg-white border border-neutral-200 rounded-xl text-xs font-bold text-neutral-700 focus:outline-none focus:border-neutral-900 transition-all"
        >
          <option value="todos">Todos los estados</option>
          <option value="pendiente">Pendiente</option>
          <option value="proximo_vencer">Próximo a vencer</option>
          <option value="vencido">Vencido</option>
          <option value="pagado">Pagado</option>
        </select>
        <select
          value={filtroTipo}
          onChange={e => setFiltroTipo(e.target.value as any)}
          className="px-4 py-2 bg-white border border-neutral-200 rounded-xl text-xs font-bold text-neutral-700 focus:outline-none focus:border-neutral-900 transition-all"
        >
          <option value="todos">Todos los tipos</option>
          {TIPOS_GASTO.map(t => (
            <option key={t.value} value={t.value}>{t.icon} {t.label}</option>
          ))}
        </select>
        {gastosFiltrados.length > 0 && (
          <span className="ml-auto text-[10px] font-black text-neutral-400 uppercase tracking-widest">
            {gastosFiltrados.length} gasto{gastosFiltrados.length !== 1 ? 's' : ''} · Total proyectado: {formatMXN(totalProyectado)}
          </span>
        )}
      </div>

      <div className="bg-neutral-50 rounded-2xl sm:rounded-[2.5rem] border border-neutral-100 p-4 sm:p-8 shadow-sm overflow-hidden">
        {loading ? (
          <div className="p-16 text-center">
            <div className="w-8 h-8 border-2 border-neutral-900 border-t-transparent rounded-full animate-spin mx-auto" />
          </div>
        ) : gastosFiltrados.length === 0 ? (
          <div className="p-16 text-center">
            <div className="w-16 h-16 bg-white rounded-2xl flex items-center justify-center mx-auto mb-4 text-3xl shadow-sm">💸</div>
            <p className="font-black text-neutral-400 text-xs uppercase tracking-widest">Sin gastos registrados</p>
            <p className="text-xs text-neutral-300 mt-1">Agrega tu primer gasto recurrente</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left">
              <thead>
                <tr className="border-b border-neutral-200">
                  {['Nombre', 'Tipo', 'Monto', 'Frecuencia', 'Próximo Pago', 'Estado', 'Acciones'].map(h => (
                    <th key={h} className="px-5 py-4 text-[9px] font-black text-neutral-400 uppercase tracking-widest whitespace-nowrap">
                      {h}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody className="divide-y divide-neutral-100">
                {gastosFiltrados.map(gasto => {
                  const colores = COLORES_ESTADO_PAGO[gasto.estado_pago as EstadoPago] ?? COLORES_ESTADO_PAGO.pendiente;
                  const tipoInfo = TIPOS_GASTO.find(t => t.value === gasto.tipo);
                  return (
                    <tr key={gasto.id} className="hover:bg-white/50 transition-colors group">
                      <td className="px-5 py-4">
                        <p className="font-bold text-neutral-900 text-sm">{gasto.nombre}</p>
                        {gasto.folio_comprobante && (
                          <p className="text-[10px] text-neutral-400 mt-0.5 font-mono">{gasto.folio_comprobante}</p>
                        )}
                      </td>
                      <td className="px-5 py-4 whitespace-nowrap">
                        <span className="text-xs font-bold text-neutral-600">
                          {tipoInfo?.icon} {tipoInfo?.label ?? gasto.tipo}
                        </span>
                      </td>
                      <td className="px-5 py-4 whitespace-nowrap">
                        <p className="font-black text-neutral-900 text-sm">{formatMXN(gasto.monto_proyectado)}</p>
                        {gasto.monto_real > 0 && (
                          <p className="text-[10px] text-neutral-400">Real: {formatMXN(gasto.monto_real)}</p>
                        )}
                      </td>
                      <td className="px-5 py-4 whitespace-nowrap">
                        <span className="text-xs font-bold text-neutral-500 capitalize">{gasto.frecuencia}</span>
                      </td>
                      <td className="px-5 py-4">
                        {gasto.proxima_fecha_pago ? (
                          <div>
                            <p className="text-xs font-bold text-neutral-700">{gasto.proxima_fecha_pago}</p>
                            {gasto.dias_para_vencer != null && (
                              <p className={`text-[10px] font-black ${gasto.dias_para_vencer <= 0 ? 'text-red-500' : gasto.dias_para_vencer <= 3 ? 'text-neutral-500' : 'text-neutral-400'}`}>
                                {gasto.dias_para_vencer <= 0 ? 'Vencido' : `En ${gasto.dias_para_vencer} días`}
                              </p>
                            )}
                          </div>
                        ) : (
                          <span className="text-xs text-neutral-300">—</span>
                        )}
                      </td>
                      <td className="px-5 py-4">
                        <span className={`px-3 py-1 rounded-xl text-[10px] font-black uppercase tracking-wide inline-flex items-center gap-1.5 ${colores.bg} ${colores.text}`}>
                          <div className={`w-1.5 h-1.5 rounded-full ${colores.dot}`} />
                          {gasto.estado_pago.replace('_', ' ')}
                        </span>
                      </td>
                      <td className="px-5 py-4">
                        <div className="flex items-center gap-2">
                          <button
                            onClick={() => abrirPago(gasto)}
                            className="px-3 py-1.5 bg-neutral-100 text-neutral-800 text-[9px] font-black rounded-lg hover:bg-neutral-900 hover:text-white transition-all uppercase tracking-widest"
                          >
                            Pagar
                          </button>
                          <button
                            onClick={() => { setEditando(gasto); setModalGasto(true); }}
                            className="p-2 text-neutral-300 hover:text-neutral-700 transition-colors rounded-xl hover:bg-neutral-100"
                            title="Editar"
                          >
                            ✏️
                          </button>
                          <button
                            onClick={() => handleEliminar(gasto)}
                            className="p-2 text-neutral-300 hover:text-red-500 transition-colors rounded-xl hover:bg-red-50"
                            title="Eliminar"
                          >
                            🗑️
                          </button>
                        </div>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {modalGasto && (
        <div className="fixed inset-0 bg-neutral-900/60 backdrop-blur-sm flex items-center justify-center z-50 p-4">
          <div className="bg-white rounded-[2.5rem] p-8 w-full max-w-lg shadow-2xl max-h-[92vh] overflow-y-auto custom-scrollbar animate-in fade-in zoom-in-95 duration-300">
            <div className="flex items-center justify-between mb-6">
              <div>
                <h3 className="text-lg font-black text-neutral-900 uppercase tracking-tight">
                  {editando ? 'Editar Gasto' : 'Nuevo Gasto'}
                </h3>
                <p className="text-[10px] text-neutral-400 uppercase tracking-widest mt-0.5">
                  {editando ? editando.nombre : 'Configura tu gasto recurrente'}
                </p>
              </div>
              <button
                onClick={() => { setModalGasto(false); setEditando(null); }}
                className="w-8 h-8 flex items-center justify-center text-neutral-400 hover:text-neutral-700 transition-colors text-lg"
              >✕</button>
            </div>

            <div className="space-y-4">
              <FormField label="Nombre del Gasto">
                <input
                  type="text"
                  value={form.nombre}
                  onChange={e => setForm(f => ({ ...f, nombre: e.target.value }))}
                  className={INPUT_CLS}
                  placeholder="Ej: Renta Local Centro"
                />
              </FormField>

              <div className="grid grid-cols-2 gap-4">
                <FormField label="Tipo">
                  <select value={form.tipo} onChange={e => setForm(f => ({ ...f, tipo: e.target.value as TipoGasto }))} className={INPUT_CLS}>
                    {TIPOS_GASTO.map(t => <option key={t.value} value={t.value}>{t.icon} {t.label}</option>)}
                  </select>
                </FormField>
                <FormField label="Categoría">
                  <select value={form.categoria} onChange={e => setForm(f => ({ ...f, categoria: e.target.value as CategoriaGasto }))} className={INPUT_CLS}>
                    {CATEGORIAS_GASTO.map(c => <option key={c.value} value={c.value}>{c.label}</option>)}
                  </select>
                </FormField>
              </div>

              <FormField label="Monto Proyectado (MXN)">
                <input
                  type="number" step="0.01" min="0"
                  value={form.monto_proyectado || ''}
                  onChange={e => setForm(f => ({ ...f, monto_proyectado: parseFloat(e.target.value) || 0 }))}
                  className={INPUT_CLS}
                  placeholder="0.00"
                />
              </FormField>

              <FormField label="Frecuencia">
                <select
                  value={form.frecuencia}
                  onChange={e => setForm(f => ({ ...f, frecuencia: e.target.value as FrecuenciaGasto, dia_pago: undefined, intervalo_dias: undefined }))}
                  className={INPUT_CLS}
                >
                  {FRECUENCIAS_GASTO.map(fr => <option key={fr.value} value={fr.value}>{fr.label}</option>)}
                </select>
              </FormField>

              {mostrarDiaSemana && (
                <FormField label="Día de la Semana">
                  <select value={form.dia_pago ?? 1} onChange={e => setForm(f => ({ ...f, dia_pago: parseInt(e.target.value) }))} className={INPUT_CLS}>
                    {DIAS_SEMANA.map(d => <option key={d.value} value={d.value}>{d.label}</option>)}
                  </select>
                </FormField>
              )}

              {mostrarDiaMes && (
                <FormField label="Día del Mes (1–28)">
                  <input
                    type="number" min={1} max={28}
                    value={form.dia_pago ?? ''}
                    onChange={e => setForm(f => ({ ...f, dia_pago: parseInt(e.target.value) || undefined }))}
                    className={INPUT_CLS}
                    placeholder="15"
                  />
                </FormField>
              )}

              {mostrarIntervalo && (
                <FormField label="Intervalo en Días">
                  <input
                    type="number" min={1}
                    value={form.intervalo_dias ?? ''}
                    onChange={e => setForm(f => ({ ...f, intervalo_dias: parseInt(e.target.value) || undefined }))}
                    className={INPUT_CLS}
                    placeholder="Ej: 45"
                  />
                </FormField>
              )}

              <div className="grid grid-cols-2 gap-4">
                <FormField label="Fecha Inicio">
                  <input type="date" value={form.fecha_inicio} onChange={e => setForm(f => ({ ...f, fecha_inicio: e.target.value }))} className={INPUT_CLS} />
                </FormField>
                <FormField label="Fecha Fin (Opcional)">
                  <input type="date" value={form.fecha_fin ?? ''} onChange={e => setForm(f => ({ ...f, fecha_fin: e.target.value || undefined }))} className={INPUT_CLS} />
                </FormField>
              </div>

              <FormField label="Folio / Comprobante (Opcional)">
                <input
                  type="text"
                  value={form.folio_comprobante ?? ''}
                  onChange={e => setForm(f => ({ ...f, folio_comprobante: e.target.value || undefined }))}
                  className={INPUT_CLS}
                  placeholder="Ej: FACT-0042"
                />
              </FormField>

              <FormField label="Notas (Opcional)">
                <textarea
                  value={form.notas ?? ''}
                  onChange={e => setForm(f => ({ ...f, notas: e.target.value || undefined }))}
                  rows={2}
                  className={`${INPUT_CLS} resize-none`}
                  placeholder="Notas adicionales..."
                />
              </FormField>
            </div>

            <div className="flex gap-3 mt-6 pt-6 border-t border-neutral-100">
              <button
                onClick={() => { setModalGasto(false); setEditando(null); }}
                className="flex-1 py-3 border border-neutral-200 rounded-2xl text-xs font-black text-neutral-500 hover:bg-neutral-50 transition-all uppercase tracking-widest"
              >
                Cancelar
              </button>
              <button
                onClick={handleGuardar}
                disabled={guardando || !form.nombre.trim() || !form.monto_proyectado}
                className="flex-1 py-3 bg-neutral-900 text-white rounded-2xl text-xs font-black uppercase tracking-widest hover:scale-[1.02] active:scale-95 transition-all disabled:opacity-40 disabled:cursor-not-allowed shadow-lg"
              >
                {guardando ? 'Guardando...' : editando ? 'Actualizar' : 'Crear Gasto'}
              </button>
            </div>
          </div>
        </div>
      )}

      {modalPago && gastoParaPago && (
        <div className="fixed inset-0 bg-neutral-900/60 backdrop-blur-sm flex items-center justify-center z-50 p-4">
          <div className="bg-white rounded-[2.5rem] p-8 w-full max-w-md shadow-2xl animate-in fade-in zoom-in-95 duration-300">
            <div className="flex items-center justify-between mb-6">
              <div>
                <h3 className="text-lg font-black text-neutral-900 uppercase tracking-tight">Registrar Pago</h3>
                <p className="text-[10px] text-neutral-400 uppercase tracking-widest mt-0.5">{gastoParaPago.nombre}</p>
              </div>
              <button onClick={() => setModalPago(false)} className="w-8 h-8 flex items-center justify-center text-neutral-400 hover:text-neutral-700 text-lg">✕</button>
            </div>

            <div className="space-y-4">
              <FormField label="Fecha de Pago">
                <input type="date" value={formPago.fecha_pago} onChange={e => setFormPago(f => ({ ...f, fecha_pago: e.target.value }))} className={INPUT_CLS} />
              </FormField>
              <FormField label="Monto Pagado (MXN)">
                <input
                  type="number" step="0.01" min="0"
                  value={formPago.monto_pagado || ''}
                  onChange={e => setFormPago(f => ({ ...f, monto_pagado: parseFloat(e.target.value) || 0 }))}
                  className={INPUT_CLS}
                  placeholder={`Proyectado: ${formatMXN(gastoParaPago.monto_proyectado)}`}
                />
              </FormField>
              <FormField label="Método de Pago">
                <select value={formPago.metodo_pago} onChange={e => setFormPago(f => ({ ...f, metodo_pago: e.target.value }))} className={INPUT_CLS}>
                  <option value="efectivo">💵 Efectivo</option>
                  <option value="transferencia">🏦 Transferencia</option>
                  <option value="tarjeta">💳 Tarjeta</option>
                </select>
              </FormField>
              <FormField label="Folio Comprobante (Opcional)">
                <input type="text" value={formPago.folio_comprobante} onChange={e => setFormPago(f => ({ ...f, folio_comprobante: e.target.value }))} className={INPUT_CLS} placeholder="Ej: REC-001" />
              </FormField>
              <FormField label="Notas (Opcional)">
                <input type="text" value={formPago.notas} onChange={e => setFormPago(f => ({ ...f, notas: e.target.value }))} className={INPUT_CLS} placeholder="Referencia o nota..." />
              </FormField>
            </div>

            <div className="flex gap-3 mt-6 pt-6 border-t border-neutral-100">
              <button onClick={() => setModalPago(false)} className="flex-1 py-3 border border-neutral-200 rounded-2xl text-xs font-black text-neutral-500 hover:bg-neutral-50 transition-all uppercase tracking-widest">
                Cancelar
              </button>
              <button
                onClick={handleRegistrarPago}
                disabled={guardando || !formPago.monto_pagado}
                className="flex-1 py-3 bg-neutral-800 text-white rounded-2xl text-xs font-black uppercase tracking-widest hover:scale-[1.02] active:scale-95 transition-all disabled:opacity-40 shadow-lg"
              >
                {guardando ? 'Registrando...' : 'Confirmar Pago'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

const INPUT_CLS = 'w-full bg-neutral-50 border border-neutral-100 px-5 py-3 rounded-2xl text-xs font-bold focus:outline-none focus:border-neutral-900 transition-all';

const FormField = ({ label, children }: { label: string; children: React.ReactNode }) => (
  <div>
    <label className="text-[9px] font-black text-neutral-400 uppercase tracking-widest ml-2 mb-1 block">{label}</label>
    {children}
  </div>
);

export default GastosManager;
