import { useState, useEffect, useMemo } from 'react';
import { useCortes } from './hooks';
import { CorteCaja, FiltrosCortes, TipoCorte, TurnoCorte, COLORES_TIPO_CORTE } from './types';
import { formatMXN, getFechaHoy, getFechaInicioPeriodo } from './utils';

const CortesManager = () => {
  const { cortes, loading, cargarCortes } = useCortes();
  const [filtros, setFiltros] = useState<FiltrosCortes>({
    fecha_inicio: getFechaInicioPeriodo('mes', getFechaHoy()),
    fecha_fin: getFechaHoy(),
  });
  const [corteSeleccionado, setCorteSeleccionado] = useState<CorteCaja | null>(null);

  useEffect(() => {
    cargarCortes(filtros);
  }, [cargarCortes]);

  const handleFiltrar = () => {
    cargarCortes(filtros);
  };

  const cortesCasted = cortes as CorteCaja[];

  const totales = useMemo(() => ({
    ventas: cortesCasted.reduce((a, c) => a + c.total_ventas, 0),
    efectivo: cortesCasted.reduce((a, c) => a + c.total_efectivo, 0),
    tarjeta: cortesCasted.reduce((a, c) => a + c.total_tarjeta, 0),
    transferencia: cortesCasted.reduce((a, c) => a + c.total_transferencia, 0),
    diferencia: cortesCasted.reduce((a, c) => a + c.diferencia, 0),
  }), [cortesCasted]);

  return (
    <div className="space-y-8 animate-in fade-in duration-500">

      <div>
        <h3 className="text-xl font-black text-neutral-900 uppercase tracking-tight">Cortes X / Z</h3>
        <p className="text-[10px] text-neutral-400 font-black uppercase tracking-widest mt-0.5">Auditoría de caja y arqueos</p>
      </div>

      <div className="bg-neutral-50 rounded-2xl sm:rounded-[2.5rem] border border-neutral-100 p-4 sm:p-8">
        <p className="text-[10px] font-black text-neutral-400 uppercase tracking-widest mb-4">Filtros de Búsqueda</p>
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-3">
          <div>
            <label className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mb-1 block">Desde</label>
            <input
              type="date"
              value={filtros.fecha_inicio ?? ''}
              onChange={e => setFiltros(f => ({ ...f, fecha_inicio: e.target.value }))}
              className="w-full bg-white border border-neutral-200 px-4 py-2.5 rounded-xl text-xs font-bold focus:outline-none focus:border-neutral-900 transition-all"
            />
          </div>
          <div>
            <label className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mb-1 block">Hasta</label>
            <input
              type="date"
              value={filtros.fecha_fin ?? ''}
              onChange={e => setFiltros(f => ({ ...f, fecha_fin: e.target.value }))}
              className="w-full bg-white border border-neutral-200 px-4 py-2.5 rounded-xl text-xs font-bold focus:outline-none focus:border-neutral-900 transition-all"
            />
          </div>
          <div>
            <label className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mb-1 block">Tipo Corte</label>
            <select
              value={filtros.tipo_corte ?? ''}
              onChange={e => setFiltros(f => ({ ...f, tipo_corte: (e.target.value as TipoCorte) || undefined }))}
              className="w-full bg-white border border-neutral-200 px-4 py-2.5 rounded-xl text-xs font-bold focus:outline-none focus:border-neutral-900 transition-all"
            >
              <option value="">Todos</option>
              <option value="X">Corte X (Parcial)</option>
              <option value="Z">Corte Z (Cierre)</option>
            </select>
          </div>
          <div>
            <label className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mb-1 block">Turno</label>
            <select
              value={filtros.turno ?? ''}
              onChange={e => setFiltros(f => ({ ...f, turno: (e.target.value as TurnoCorte) || undefined }))}
              className="w-full bg-white border border-neutral-200 px-4 py-2.5 rounded-xl text-xs font-bold focus:outline-none focus:border-neutral-900 transition-all"
            >
              <option value="">Todos</option>
              <option value="matutino">🌅 Matutino</option>
              <option value="vespertino">☀️ Vespertino</option>
              <option value="nocturno">🌙 Nocturno</option>
            </select>
          </div>
          <div>
            <label className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mb-1 block">&nbsp;</label>
            <button
              onClick={handleFiltrar}
              className="w-full py-2.5 bg-neutral-900 text-white text-[10px] font-black uppercase tracking-widest rounded-xl hover:scale-[1.02] active:scale-95 transition-all"
            >
              Buscar
            </button>
          </div>
        </div>
      </div>

      {cortesCasted.length > 0 && (
        <div className="grid grid-cols-2 sm:grid-cols-5 gap-4">
          {[
            { label: 'TOTAL VENTAS', value: formatMXN(totales.ventas), color: 'bg-neutral-900 text-white shadow-2xl shadow-neutral-200' },
            { label: 'EFECTIVO', value: formatMXN(totales.efectivo), color: 'bg-neutral-50' },
            { label: 'TARJETA', value: formatMXN(totales.tarjeta), color: 'bg-neutral-50' },
            { label: 'TRANSFERENCIA', value: formatMXN(totales.transferencia), color: 'bg-neutral-50' },
            {
              label: 'DIFERENCIA TOTAL',
              value: formatMXN(totales.diferencia),
              color: totales.diferencia >= 0 ? 'bg-neutral-50' : 'bg-red-50',
            },
          ].map(t => (
            <div key={t.label} className={`${t.color} p-4 sm:p-6 rounded-2xl sm:rounded-[2rem] border border-neutral-100 relative overflow-hidden group hover:scale-[1.02] transition-all duration-300`}>
              <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">{t.label}</p>
              <p className={`text-base font-black mt-1 ${t.color.includes('white') ? 'text-white' : 'text-neutral-900'}`}>{t.value}</p>
            </div>
          ))}
        </div>
      )}

      <div className="bg-neutral-50 rounded-2xl sm:rounded-[2.5rem] border border-neutral-100 p-4 sm:p-8 shadow-sm overflow-hidden">
        {loading ? (
          <div className="p-16 text-center">
            <div className="w-8 h-8 border-2 border-neutral-900 border-t-transparent rounded-full animate-spin mx-auto" />
          </div>
        ) : cortesCasted.length === 0 ? (
          <div className="p-16 text-center">
            <div className="w-16 h-16 bg-white rounded-2xl flex items-center justify-center mx-auto mb-4 text-3xl shadow-sm">🧾</div>
            <p className="font-black text-neutral-400 text-xs uppercase tracking-widest">Sin cortes en este período</p>
            <p className="text-xs text-neutral-300 mt-1">Ajusta los filtros o crea un corte desde la vista de ventas</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left">
              <thead>
                <tr className="border-b border-neutral-200">
                  {['Tipo', 'Cajero', 'Turno', 'Apertura', 'Cierre', 'Ventas', 'Efectivo', 'Diferencia', 'Estado', ''].map(h => (
                    <th key={h} className="px-5 py-4 text-[9px] font-black text-neutral-400 uppercase tracking-widest whitespace-nowrap">{h}</th>
                  ))}
                </tr>
              </thead>
              <tbody className="divide-y divide-neutral-100">
                {cortesCasted.map(corte => {
                  const tipoColor = COLORES_TIPO_CORTE[corte.tipo_corte as TipoCorte] ?? COLORES_TIPO_CORTE.Z;
                  const difPositiva = corte.diferencia >= 0;
                  return (
                    <tr key={corte.id} className="hover:bg-white/50 transition-colors cursor-pointer" onClick={() => setCorteSeleccionado(corte)}>
                      <td className="px-5 py-4">
                        <span className={`px-3 py-1 rounded-lg text-[10px] font-black uppercase tracking-wide ${tipoColor.bg} ${tipoColor.text}`}>
                          {corte.tipo_corte}
                        </span>
                      </td>
                      <td className="px-5 py-4">
                        <p className="text-sm font-bold text-neutral-900">{corte.usuario_nombre}</p>
                      </td>
                      <td className="px-5 py-4">
                        <span className="text-xs font-bold text-neutral-500 capitalize">{corte.turno ?? '—'}</span>
                      </td>
                      <td className="px-5 py-4 whitespace-nowrap">
                        <p className="text-xs font-bold text-neutral-700">{formatFecha(corte.fecha_apertura)}</p>
                      </td>
                      <td className="px-5 py-4 whitespace-nowrap">
                        <p className="text-xs font-bold text-neutral-700">{corte.fecha_cierre ? formatFecha(corte.fecha_cierre) : '—'}</p>
                      </td>
                      <td className="px-5 py-4 whitespace-nowrap">
                        <p className="text-sm font-black text-neutral-900">{formatMXN(corte.total_ventas)}</p>
                      </td>
                      <td className="px-5 py-4 whitespace-nowrap">
                        <p className="text-sm font-bold text-neutral-800">{formatMXN(corte.total_efectivo)}</p>
                      </td>
                      <td className="px-5 py-4 whitespace-nowrap">
                        <p className={`text-sm font-black ${difPositiva ? 'text-neutral-800' : 'text-red-600'}`}>
                          {difPositiva ? '+' : ''}{formatMXN(corte.diferencia)}
                        </p>
                      </td>
                      <td className="px-5 py-4">
                        <span className={`px-3 py-1 rounded-xl text-[10px] font-black uppercase tracking-wide ${
                          corte.estado === 'cerrado' ? 'bg-neutral-900 text-white' : 'bg-neutral-100 text-neutral-800'
                        }`}>
                          {corte.estado}
                        </span>
                      </td>
                      <td className="px-5 py-4">
                        <button className="p-2 text-neutral-400 hover:text-neutral-900 transition-colors rounded-xl hover:bg-neutral-100 text-sm" onClick={() => setCorteSeleccionado(corte)}>
                          👁️
                        </button>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {corteSeleccionado && (
        <DetalleCorte corte={corteSeleccionado} onCerrar={() => setCorteSeleccionado(null)} />
      )}
    </div>
  );
};

const DetalleCorte = ({ corte, onCerrar }: { corte: CorteCaja; onCerrar: () => void }) => (
  <div className="fixed inset-0 bg-neutral-900/60 backdrop-blur-sm flex items-center justify-center z-50 p-4">
    <div className="bg-white rounded-[2.5rem] p-8 w-full max-w-md shadow-2xl animate-in fade-in zoom-in-95 duration-300">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h3 className="text-lg font-black text-neutral-900 uppercase tracking-tight">
            Corte {corte.tipo_corte} #{corte.id}
          </h3>
          <p className="text-[10px] text-neutral-400 uppercase tracking-widest mt-0.5">{corte.usuario_nombre} · {corte.turno ?? 'Sin turno'}</p>
        </div>
        <button onClick={onCerrar} className="w-8 h-8 flex items-center justify-center text-neutral-400 hover:text-neutral-700 text-lg">✕</button>
      </div>

      <div className="space-y-3">
        <DetalleRow label="Apertura" value={formatFecha(corte.fecha_apertura)} />
        <DetalleRow label="Cierre" value={corte.fecha_cierre ? formatFecha(corte.fecha_cierre) : 'Sin cerrar'} />
        <DetalleRow label="Fondo Inicial" value={formatMXN(corte.monto_inicial)} />

        <div className="h-px bg-neutral-100 my-2" />

        <DetalleRow label="Total Ventas" value={formatMXN(corte.total_ventas)} bold />
        <DetalleRow label="💵 Efectivo" value={formatMXN(corte.total_efectivo)} />
        <DetalleRow label="💳 Tarjeta" value={formatMXN(corte.total_tarjeta)} />
        <DetalleRow label="🏦 Transferencia" value={formatMXN(corte.total_transferencia)} />

        <div className="h-px bg-neutral-100 my-2" />

        <DetalleRow label="Entradas Manuales" value={formatMXN(corte.entradas_manuales)} />
        <DetalleRow label="Retiros Manuales" value={formatMXN(corte.retiros_manuales)} />

        <div className="h-px bg-neutral-100 my-2" />

        <DetalleRow
          label="Diferencia de Caja"
          value={`${corte.diferencia >= 0 ? '+' : ''}${formatMXN(corte.diferencia)}`}
          bold
          color={corte.diferencia >= 0 ? 'text-neutral-800' : 'text-red-600'}
        />

        {corte.observaciones && (
          <div className="mt-3 p-3 bg-neutral-50 rounded-2xl">
            <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mb-1">Observaciones</p>
            <p className="text-xs text-neutral-700">{corte.observaciones}</p>
          </div>
        )}
      </div>

      <button
        onClick={onCerrar}
        className="w-full mt-6 py-3 border border-neutral-200 rounded-2xl text-xs font-black text-neutral-500 hover:bg-neutral-50 transition-all uppercase tracking-widest"
      >
        Cerrar
      </button>
    </div>
  </div>
);

const DetalleRow = ({ label, value, bold, color }: { label: string; value: string; bold?: boolean; color?: string }) => (
  <div className="flex items-center justify-between">
    <span className="text-xs text-neutral-500 font-bold">{label}</span>
    <span className={`text-sm ${bold ? 'font-black' : 'font-bold'} ${color ?? 'text-neutral-900'}`}>{value}</span>
  </div>
);

const formatFecha = (raw: string) => {
  try { return new Date(raw).toLocaleDateString('es-MX', { day: '2-digit', month: 'short', year: 'numeric', hour: '2-digit', minute: '2-digit' }); }
  catch { return raw; }
};

export default CortesManager;
