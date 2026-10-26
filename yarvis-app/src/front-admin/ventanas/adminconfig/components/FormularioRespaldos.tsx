// ═══════════════════════════════════════════════════════════════════════════
// FORMULARIO DE RESPALDOS — Carpeta/USB, programación, cifrado y
// restauración con prueba real. La contraseña nunca se muestra ni se
// registra en el frontend; viaja solo al comando que la necesita.
// ═══════════════════════════════════════════════════════════════════════════

import type { InfoRespaldo } from "../../../../services/respaldos";
import { useRespaldos } from "../hooks/useRespaldos";

/** Tamaño legible; nunca produce "NaN MB" con datos basura. */
export function formatearBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 B";
  const unidades = ["B", "KB", "MB", "GB"];
  const i = Math.min(unidades.length - 1, Math.floor(Math.log(bytes) / Math.log(1024)));
  const valor = bytes / 1024 ** i;
  // Enteros sin ".0" (1024 -> "1 KB"); los decimales conservan 1 cifra
  // y los valores grandes se redondean para no fingir precisión.
  const texto = Number.isInteger(valor)
    ? valor.toString()
    : valor >= 10
      ? Math.round(valor).toString()
      : valor.toFixed(1);
  return `${texto} ${unidades[i]}`;
}

const FormularioRespaldos = () => {
  const r = useRespaldos();
  const trabajando = r.ocupado !== null;

  return (
    <section
      className="bg-neutral-50 p-8 rounded-[2.5rem] border border-neutral-100 space-y-6 shadow-sm"
      aria-labelledby="titulo-respaldos"
    >
      <header className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h3 id="titulo-respaldos" className="text-[10px] font-black text-neutral-400 uppercase tracking-[0.4em]">
            Respaldo de la Caja
          </h3>
          <p className="mt-2 text-xs text-neutral-500 max-w-2xl">
            Copia cifrada fuera de este disco (USB o carpeta de red). Sin la contraseña
            el archivo es ilegible, incluso para YARVIS.
          </p>
        </div>
        <span className={`text-[9px] font-black px-3 py-1 rounded-full tracking-widest ${
          r.cargando ? "bg-neutral-200 text-neutral-500"
            : r.cfg.activo ? "bg-emerald-100 text-emerald-700" : "bg-amber-100 text-amber-700"
        }`}>
          {r.cargando ? "CARGANDO" : r.cfg.activo ? "ACTIVO" : "APAGADO"}
        </span>
      </header>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
        <div className="lg:col-span-2 space-y-2">
          <label htmlFor="respaldo-destino" className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">
            Carpeta o USB
          </label>
          <div className="flex gap-2">
            <input
              id="respaldo-destino"
              aria-label="Carpeta de respaldos"
              readOnly
              value={r.cfg.destino ?? ""}
              placeholder="Sin carpeta elegida"
              className="flex-1 bg-white border border-neutral-200 rounded-xl px-4 py-3 text-xs font-mono text-neutral-600 truncate"
            />
            <button
              type="button"
              onClick={r.elegirDestino}
              disabled={trabajando}
              className="px-4 py-3 rounded-xl bg-neutral-900 text-white text-[9px] font-black uppercase tracking-widest hover:bg-neutral-700 disabled:opacity-40"
            >
              Elegir
            </button>
          </div>
        </div>

        <label className="flex items-center gap-3 bg-white border border-neutral-200 rounded-xl px-4 py-3 cursor-pointer">
          <input
            type="checkbox"
            checked={r.cfg.activo}
            onChange={(e) => r.setCfg({ ...r.cfg, activo: e.target.checked })}
            className="h-4 w-4 accent-neutral-900"
          />
          <span className="text-[10px] font-black uppercase tracking-widest text-neutral-600">
            Respaldo automático
          </span>
        </label>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <label className="space-y-2 block">
          <span className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">Cada (horas)</span>
          <input
            type="number" min={1} max={168}
            aria-label="Horas entre respaldos"
            value={r.cfg.cada_horas}
            onChange={(e) => r.setCfg({ ...r.cfg, cada_horas: Number(e.target.value) })}
            className="w-full bg-white border border-neutral-200 rounded-xl px-4 py-3 text-xs font-mono"
          />
        </label>
        <label className="space-y-2 block">
          <span className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">Conservar (respaldos)</span>
          <input
            type="number" min={1} max={100}
            aria-label="Respaldos a conservar"
            value={r.cfg.retener}
            onChange={(e) => r.setCfg({ ...r.cfg, retener: Number(e.target.value) })}
            className="w-full bg-white border border-neutral-200 rounded-xl px-4 py-3 text-xs font-mono"
          />
        </label>
      </div>

      <div className="space-y-2">
        <label htmlFor="respaldo-pass" className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">
          Contraseña del respaldo
        </label>
        <input
          id="respaldo-pass"
          type="password"
          autoComplete="new-password"
          value={r.passphrase}
          onChange={(e) => r.setPassphrase(e.target.value)}
          placeholder={r.cfg.tiene_passphrase
            ? "Ya hay una guardada (déjala vacía para conservarla)"
            : "Mínimo 8 caracteres"}
          className="w-full bg-white border border-neutral-200 rounded-xl px-4 py-3 text-xs font-mono"
        />
        <p className="text-[10px] text-neutral-400">
          {r.cfg.tiene_passphrase
            ? "Contraseña guardada en este equipo con protección del sistema."
            : "Sin contraseña no se pueden activar los respaldos automáticos."}
        </p>
      </div>

      {r.aviso && (
        <p role="alert" className="text-xs font-bold text-amber-700 bg-amber-50 border border-amber-200 rounded-xl px-4 py-3">
          {r.aviso}
        </p>
      )}

      <div className="flex flex-wrap gap-3">
        <button
          type="button"
          onClick={r.guardar}
          disabled={trabajando || r.cargando}
          className="px-5 py-3 rounded-xl bg-neutral-900 text-white text-[9px] font-black uppercase tracking-widest hover:bg-neutral-700 disabled:opacity-40"
        >
          {r.ocupado === "guardar" ? "Guardando..." : "Guardar configuración"}
        </button>
        <button
          type="button"
          onClick={r.crearAhora}
          disabled={trabajando || r.cargando || !r.cfg.destino}
          className="px-5 py-3 rounded-xl bg-emerald-600 text-white text-[9px] font-black uppercase tracking-widest hover:bg-emerald-700 disabled:opacity-40"
        >
          {r.ocupado === "crear" ? "Respaldando..." : "Respaldar ahora"}
        </button>
      </div>

      <div className="border-t border-neutral-200 pt-5 space-y-3">
        <div className="flex flex-wrap items-baseline justify-between gap-2">
          <h4 className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">
            Respaldos en la carpeta
          </h4>
          <p className="text-[10px] text-neutral-400">
            {r.cfg.ultimo_exito ? `Último: ${r.cfg.ultimo_exito}` : "Aún no se ha creado ninguno"}
          </p>
        </div>

        {r.lista.length === 0 ? (
          <p className="text-xs text-neutral-400">
            {r.cfg.destino
              ? "No hay respaldos .yarvisbak en esa carpeta."
              : "Elige una carpeta para ver o crear respaldos."}
          </p>
        ) : (
          <ul className="space-y-2">
            {r.lista.map((item: InfoRespaldo) => (
              <li key={item.archivo} className="bg-white border border-neutral-200 rounded-xl px-4 py-3 space-y-2">
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <div className="min-w-0">
                    <p className="text-xs font-mono font-bold text-neutral-700 truncate">{item.nombre}</p>
                    <p className="text-[10px] text-neutral-400">
                      {item.fecha || "sin fecha"} · {formatearBytes(item.bytes)}
                    </p>
                  </div>
                  {r.porRestaurar !== item.archivo && (
                    <div className="flex gap-2">
                      <button
                        type="button"
                        onClick={() => r.probar(item)}
                        disabled={trabajando}
                        className="px-3 py-2 rounded-lg bg-neutral-100 text-neutral-600 text-[9px] font-black uppercase tracking-widest hover:bg-neutral-200 disabled:opacity-40"
                      >
                        {r.ocupado === `probar:${item.archivo}` ? "Probando..." : "Probar"}
                      </button>
                      <button
                        type="button"
                        onClick={() => r.setPorRestaurar(item.archivo)}
                        disabled={trabajando}
                        className="px-3 py-2 rounded-lg bg-red-50 text-red-700 text-[9px] font-black uppercase tracking-widest hover:bg-red-100 disabled:opacity-40"
                      >
                        Restaurar
                      </button>
                    </div>
                  )}
                </div>

                {r.porRestaurar === item.archivo && (
                  <div className="bg-red-50 border border-red-200 rounded-lg px-3 py-3 space-y-2">
                    <p className="text-[11px] text-red-800 font-bold">
                      Esto reemplaza la base actual y reinicia la caja. Antes se guarda una
                      copia de seguridad de la base actual.
                    </p>
                    <div className="flex gap-2">
                      <button
                        type="button"
                        onClick={r.confirmarRestaurar}
                        disabled={trabajando}
                        className="px-3 py-2 rounded-lg bg-red-700 text-white text-[9px] font-black uppercase tracking-widest hover:bg-red-800 disabled:opacity-40"
                      >
                        {r.ocupado === "restaurar" ? "Restaurando..." : "Sí, restaurar"}
                      </button>
                      <button
                        type="button"
                        onClick={() => r.setPorRestaurar(null)}
                        disabled={trabajando}
                        className="px-3 py-2 rounded-lg bg-white text-neutral-600 text-[9px] font-black uppercase tracking-widest border border-neutral-200"
                      >
                        Cancelar
                      </button>
                    </div>
                  </div>
                )}
              </li>
            ))}
          </ul>
        )}

        {r.prueba && (
          <div className={`rounded-xl px-4 py-3 text-xs border ${
            r.prueba.ok
              ? "bg-emerald-50 border-emerald-200 text-emerald-800"
              : "bg-red-50 border-red-200 text-red-800"
          }`} role="status">
            <p className="font-bold">{r.prueba.mensaje}</p>
            {r.prueba.ok && (
              <p className="text-[10px] mt-1">
                {r.prueba.ventas} ventas · {r.prueba.productos} productos · {r.prueba.usuarios} usuarios ·
                esquema v{r.prueba.version_esquema}
              </p>
            )}
          </div>
        )}
      </div>
    </section>
  );
};

export default FormularioRespaldos;
