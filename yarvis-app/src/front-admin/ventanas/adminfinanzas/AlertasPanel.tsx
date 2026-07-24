import { useState, useEffect, useMemo } from 'react';
import { useAlertas } from './hooks';
import { AlertaFinanciera, SeveridadAlerta, COLORES_SEVERIDAD } from './types';

const AlertasPanel = () => {
  const { alertas, loading, cargarAlertas, marcarLeida, generarAlertas } = useAlertas();
  const [soloNoLeidas, setSoloNoLeidas] = useState(false);
  const [generando, setGenerando] = useState(false);
  const [successMsg, setSuccessMsg] = useState('');

  useEffect(() => {
    cargarAlertas(soloNoLeidas);
  }, [soloNoLeidas, cargarAlertas]);

  const alertasCasted = alertas as AlertaFinanciera[];

  const agrupadas = useMemo(() => ({
    rojo: alertasCasted.filter(a => a.severidad === 'rojo'),
    amarillo: alertasCasted.filter(a => a.severidad === 'amarillo'),
    verde: alertasCasted.filter(a => a.severidad === 'verde'),
  }), [alertasCasted]);

  const noLeidas = useMemo(() => alertasCasted.filter(a => !a.leida).length, [alertasCasted]);

  const toast = (msg: string) => {
    setSuccessMsg(msg);
    setTimeout(() => setSuccessMsg(''), 3000);
  };

  const handleGenerarAlertas = async () => {
    setGenerando(true);
    try {
      await generarAlertas();
      await cargarAlertas(soloNoLeidas);
      toast('Alertas actualizadas correctamente');
    } catch (e) {
      console.error(e);
    } finally {
      setGenerando(false);
    }
  };

  const handleMarcarLeida = async (id: number) => {
    await marcarLeida(id);
    await cargarAlertas(soloNoLeidas);
  };

  return (
    <div className="space-y-8 animate-in fade-in duration-500">

      <div className="relative flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h3 className="text-xl font-black text-neutral-900 uppercase tracking-tight">Sistema de Alertas</h3>
          <p className="text-[10px] text-neutral-400 font-black uppercase tracking-widest mt-0.5">Semáforo financiero y vencimientos</p>
        </div>
        {successMsg && (
          <div className="absolute top-0 right-44 bg-neutral-100 border border-neutral-300 text-neutral-800 px-4 py-2 rounded-2xl text-[10px] font-black uppercase tracking-widest animate-in fade-in slide-in-from-top-2 duration-300 shadow">
            {successMsg}
          </div>
        )}
        <div className="flex flex-wrap items-center gap-3">
          <label className="flex items-center gap-2 cursor-pointer select-none">
            <div
              onClick={() => setSoloNoLeidas(v => !v)}
              className={`w-10 h-5 rounded-full transition-all relative ${soloNoLeidas ? 'bg-neutral-900' : 'bg-neutral-200'}`}
            >
              <div className={`absolute top-0.5 w-4 h-4 bg-white rounded-full shadow transition-all ${soloNoLeidas ? 'left-5' : 'left-0.5'}`} />
            </div>
            <span className="text-[10px] font-black text-neutral-600 uppercase tracking-widest">Solo no leídas</span>
          </label>
          <button
            onClick={handleGenerarAlertas}
            disabled={generando}
            className="px-4 py-2.5 bg-neutral-900 text-white text-[10px] font-black uppercase tracking-widest rounded-2xl hover:scale-[1.02] active:scale-95 transition-all shadow-lg disabled:opacity-50"
          >
            {generando ? 'Generando...' : '⚡ Generar Alertas'}
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-3 gap-6">
        {(
          [
            { sev: 'rojo', label: 'CRÍTICO', alertas: agrupadas.rojo, emoji: '🔴', grad: 'bg-neutral-900 text-white shadow-2xl shadow-neutral-200' },
            { sev: 'amarillo', label: 'ADVERTENCIA', alertas: agrupadas.amarillo, emoji: '🟡', grad: 'bg-neutral-50' },
            { sev: 'verde', label: 'OK', alertas: agrupadas.verde, emoji: '⚪', grad: 'bg-neutral-50' },
          ] as const
        ).map(s => (
          <div key={s.sev} className={`${s.grad} p-4 sm:p-8 rounded-2xl sm:rounded-[2rem] border border-neutral-100 relative overflow-hidden group hover:scale-[1.02] transition-all duration-300`}>
            <div className="relative z-10">
              <p className={`text-[9px] sm:text-[10px] font-black uppercase tracking-widest mb-2 ${s.grad.includes('white') ? 'text-neutral-400' : 'text-neutral-400'}`}>{s.label}</p>
              <h3 className="text-4xl sm:text-6xl font-black leading-none">{s.alertas.length}</h3>
              <span className={`absolute top-4 sm:top-8 right-4 sm:right-8 text-[7px] sm:text-[8px] font-black px-1.5 sm:px-2 py-0.5 sm:py-1 rounded-lg uppercase ${s.grad.includes('white') ? 'bg-white/10 text-neutral-400' : 'bg-neutral-100 text-neutral-400'}`}>
                {s.alertas.filter(a => !a.leida).length} sin leer
              </span>
            </div>
            <span className="absolute right-4 bottom-3 text-5xl opacity-10 select-none">{s.emoji}</span>
          </div>
        ))}
      </div>

      {noLeidas > 0 && (
        <div className="bg-neutral-50 border border-neutral-200 rounded-2xl p-4 flex items-center justify-between">
          <p className="text-xs font-bold text-neutral-800">
            Tienes <span className="font-black">{noLeidas}</span> alerta{noLeidas !== 1 ? 's' : ''} sin leer
          </p>
          <button
            onClick={async () => {
              const sinLeer = alertasCasted.filter(a => !a.leida);
              await Promise.all(sinLeer.map(a => marcarLeida(a.id)));
              await cargarAlertas(soloNoLeidas);
              toast('Todas las alertas marcadas como leídas');
            }}
            className="text-[10px] font-black text-neutral-800 uppercase tracking-widest hover:underline"
          >
            Marcar todas como leídas
          </button>
        </div>
      )}

      {loading ? (
        <div className="p-16 text-center">
          <div className="w-8 h-8 border-2 border-neutral-900 border-t-transparent rounded-full animate-spin mx-auto" />
        </div>
      ) : alertasCasted.length === 0 ? (
        <div className="bg-neutral-50 rounded-2xl sm:rounded-[2.5rem] border border-neutral-100 p-4 sm:p-8 shadow-sm text-center">
          <div className="w-16 h-16 bg-white rounded-2xl flex items-center justify-center mx-auto mb-4 text-3xl shadow-sm">⚪</div>
          <p className="font-black text-neutral-400 text-xs uppercase tracking-widest">Sin alertas</p>
          <p className="text-xs text-neutral-300 mt-1">
            {soloNoLeidas ? 'No hay alertas pendientes de leer' : 'Genera alertas automáticas para comenzar'}
          </p>
        </div>
      ) : (
        <div className="space-y-8">
          {(['rojo', 'amarillo', 'verde'] as SeveridadAlerta[]).map(sev => {
            const grupo = agrupadas[sev];
            if (grupo.length === 0) return null;
            return (
              <div key={sev} className="bg-neutral-50 rounded-2xl sm:rounded-[2.5rem] border border-neutral-100 p-4 sm:p-8 shadow-sm overflow-hidden">
                <div className="flex items-center justify-between mb-4 sm:mb-6">
                  <div className="flex items-center gap-3">
                    <span className="w-1.5 h-4 bg-neutral-900 rounded-full inline-block" />
                    <span className="text-sm font-black text-neutral-900 uppercase tracking-tight">
                      {sev === 'rojo' ? 'CRÍTICO' : sev === 'amarillo' ? 'ADVERTENCIA' : 'OK'}
                    </span>
                  </div>
                  <span className="px-3 py-1 rounded-xl text-[10px] font-black bg-neutral-200 text-neutral-700">
                    {grupo.length} alerta{grupo.length !== 1 ? 's' : ''}
                  </span>
                </div>
                <div className="space-y-2">
                  {grupo.map(alerta => (
                    <AlertaRow key={alerta.id} alerta={alerta} onLeer={handleMarcarLeida} />
                  ))}
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};

const AlertaRow = ({ alerta, onLeer }: { alerta: AlertaFinanciera; onLeer: (id: number) => void }) => {
  const colores = COLORES_SEVERIDAD[alerta.severidad];
  return (
    <div className={`p-4 rounded-xl border-l-4 ${colores.border} bg-white flex items-start gap-4 transition-colors ${alerta.leida ? 'opacity-50' : 'hover:bg-neutral-50'}`}>
      <span className="text-xl leading-none mt-0.5 shrink-0">{colores.icon}</span>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-0.5">
          <p className="text-sm font-black text-neutral-900">{alerta.titulo}</p>
          {!alerta.leida && (
            <span className="w-2 h-2 bg-red-500 rounded-full shrink-0" />
          )}
        </div>
        <p className="text-xs text-neutral-500">{alerta.mensaje}</p>
        <div className="flex items-center gap-3 mt-1.5">
          <span className="text-[10px] text-neutral-400 font-mono">
            {new Date(alerta.creada_en).toLocaleDateString('es-MX', { day: '2-digit', month: 'short', hour: '2-digit', minute: '2-digit' })}
          </span>
          {alerta.fecha_vencimiento && (
            <span className="text-[10px] font-black text-neutral-600">
              Vence: {alerta.fecha_vencimiento}
            </span>
          )}
          <TipoTag tipo={alerta.tipo} />
        </div>
      </div>
      {!alerta.leida && (
        <button
          onClick={() => onLeer(alerta.id)}
          className="px-3 py-1.5 bg-neutral-100 text-neutral-600 text-[9px] font-black uppercase tracking-widest rounded-lg hover:bg-neutral-900 hover:text-white transition-all shrink-0"
        >
          Marcar leída
        </button>
      )}
    </div>
  );
};

const TipoTag = ({ tipo }: { tipo: string }) => {
  const labels: Record<string, { label: string; cls: string }> = {
    gasto_vencimiento: { label: 'Gasto', cls: 'bg-neutral-100 text-neutral-700' },
    corte_pendiente: { label: 'Corte', cls: 'bg-neutral-200 text-neutral-800' },
    stock_bajo: { label: 'Stock', cls: 'bg-neutral-100 text-neutral-700' },
    diferencia_caja: { label: 'Caja', cls: 'bg-red-100 text-red-700' },
  };
  const info = labels[tipo] ?? { label: tipo, cls: 'bg-neutral-100 text-neutral-600' };
  return (
    <span className={`px-2 py-0.5 rounded-lg text-[9px] font-black uppercase tracking-widest ${info.cls}`}>
      {info.label}
    </span>
  );
};

export default AlertasPanel;
