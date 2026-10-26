// ═══════════════════════════════════════════════════════════════════════════
// HOOK DE RESPALDOS — Estado del wizard y normalización defensiva.
//
// La UI nunca confía en la forma de lo que devuelve Tauri: un mock viejo,
// una versión distinta o un JSON raro no deben tumbar Configuración.
// ═══════════════════════════════════════════════════════════════════════════

import { useCallback, useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { notificarExito } from "../../../../components/notificaciones";
import { reportarError } from "../../../../services/tauri";
import {
  crearRespaldoAhora,
  guardarConfigRespaldo,
  leerConfigRespaldo,
  listarRespaldos,
  probarRespaldo,
  restaurarRespaldo,
  type ConfigRespaldo,
  type EntradaRespaldo,
  type InfoRespaldo,
  type PruebaRespaldo,
} from "../../../../services/respaldos";

export const CFG_DEFECTO: ConfigRespaldo = {
  activo: false,
  destino: null,
  cada_horas: 24,
  retener: 30,
  ultimo_exito: null,
  tiene_passphrase: false,
};

function enteroEntre(valor: unknown, min: number, max: number, defecto: number): number {
  const n = typeof valor === "number" ? valor : Number(valor);
  if (!Number.isFinite(n)) return defecto;
  return Math.min(max, Math.max(min, Math.trunc(n)));
}

/** Convierte cualquier respuesta (incluido el "ok" de mocks viejos). */
export function normalizarConfig(dato: unknown): ConfigRespaldo {
  if (!dato || typeof dato !== "object") return { ...CFG_DEFECTO };
  const d = dato as Record<string, unknown>;
  const destino = typeof d.destino === "string" && d.destino.trim() ? d.destino : null;
  return {
    activo: d.activo === true,
    destino,
    cada_horas: enteroEntre(d.cada_horas, 1, 168, CFG_DEFECTO.cada_horas),
    retener: enteroEntre(d.retener, 1, 100, CFG_DEFECTO.retener),
    ultimo_exito: typeof d.ultimo_exito === "string" ? d.ultimo_exito : null,
    tiene_passphrase: d.tiene_passphrase === true,
  };
}

/** Filtra basura y entradas sin ruta; la lista siempre se puede pintar. */
export function normalizarLista(dato: unknown): InfoRespaldo[] {
  if (!Array.isArray(dato)) return [];
  return dato.flatMap((item) => {
    if (!item || typeof item !== "object") return [];
    const d = item as Record<string, unknown>;
    if (typeof d.archivo !== "string" || !d.archivo) return [];
    return [{
      archivo: d.archivo,
      nombre: typeof d.nombre === "string" && d.nombre ? d.nombre : d.archivo,
      bytes: typeof d.bytes === "number" && d.bytes >= 0 ? d.bytes : 0,
      fecha: typeof d.fecha === "string" ? d.fecha : "",
    }];
  });
}

export function normalizarPrueba(dato: unknown): PruebaRespaldo {
  if (!dato || typeof dato !== "object") {
    return { ok: false, mensaje: "El backend no devolvió una prueba válida", tablas: 0,
      usuarios: 0, ventas: 0, productos: 0, detalle_ventas: 0, version_esquema: 0 };
  }
  const d = dato as Record<string, unknown>;
  const num = (v: unknown) => (typeof v === "number" && Number.isFinite(v) ? v : 0);
  return {
    ok: d.ok === true,
    mensaje: typeof d.mensaje === "string" ? d.mensaje : "Sin detalle",
    tablas: num(d.tablas), usuarios: num(d.usuarios), ventas: num(d.ventas),
    productos: num(d.productos), detalle_ventas: num(d.detalle_ventas),
    version_esquema: num(d.version_esquema),
  };
}

export function useRespaldos() {
  const [cfg, setCfg] = useState<ConfigRespaldo>(CFG_DEFECTO);
  const [lista, setLista] = useState<InfoRespaldo[]>([]);
  const [passphrase, setPassphrase] = useState("");
  const [prueba, setPrueba] = useState<PruebaRespaldo | null>(null);
  const [ocupado, setOcupado] = useState<string | null>(null);
  const [cargando, setCargando] = useState(true);
  const [porRestaurar, setPorRestaurar] = useState<string | null>(null);
  const [aviso, setAviso] = useState("");

  const cargarLista = useCallback(async (destino: string | null) => {
    if (!destino) {
      setLista([]);
      return;
    }
    try {
      setLista(normalizarLista(await listarRespaldos()));
    } catch (error) {
      reportarError("No se pudieron listar los respaldos", error);
    }
  }, []);

  useEffect(() => {
    let vivo = true;
    (async () => {
      try {
        const config = normalizarConfig(await leerConfigRespaldo());
        if (!vivo) return;
        setCfg(config);
        await cargarLista(config.destino);
      } catch (error) {
        if (vivo) reportarError("No se pudo leer la configuración de respaldos", error);
      } finally {
        if (vivo) setCargando(false);
      }
    })();
    return () => { vivo = false; };
  }, [cargarLista]);

  const elegirDestino = useCallback(async () => {
    try {
      const elegida = await open({
        directory: true,
        multiple: false,
        title: "Elige una carpeta o USB para los respaldos",
      });
      if (typeof elegida === "string" && elegida.trim()) {
        setCfg((actual) => ({ ...actual, destino: elegida }));
        setAviso("");
      }
    } catch (error) {
      reportarError("No se pudo abrir el selector de carpeta", error);
    }
  }, []);

  const guardar = useCallback(async () => {
    setAviso("");
    if (cfg.activo && !cfg.destino) {
      setAviso("Elige una carpeta o USB antes de activar el respaldo automático.");
      return;
    }
    if (cfg.activo && !cfg.tiene_passphrase && passphrase.trim().length < 8) {
      setAviso("Escribe una contraseña de al menos 8 caracteres para el respaldo.");
      return;
    }
    setOcupado("guardar");
    try {
      const entrada: EntradaRespaldo = {
        activo: cfg.activo,
        destino: cfg.destino,
        cada_horas: cfg.cada_horas,
        retener: cfg.retener,
        ...(passphrase.trim() ? { passphrase: passphrase.trim() } : {}),
      };
      const nueva = normalizarConfig(await guardarConfigRespaldo(entrada));
      setCfg(nueva);
      setPassphrase("");
      await cargarLista(nueva.destino);
      notificarExito("Configuración de respaldos guardada");
    } catch (error) {
      reportarError("No se pudo guardar la configuración de respaldos", error);
    } finally {
      setOcupado(null);
    }
  }, [cfg, passphrase, cargarLista]);

  const crearAhora = useCallback(async () => {
    setAviso("");
    if (!cfg.destino) {
      setAviso("Elige una carpeta o USB para guardar el respaldo.");
      return;
    }
    setOcupado("crear");
    try {
      const info = await crearRespaldoAhora(passphrase.trim() || undefined);
      if (info && typeof info.archivo === "string") {
        notificarExito("Respaldo cifrado creado");
        setCfg((actual) => ({ ...actual, ultimo_exito: info.fecha || actual.ultimo_exito }));
      }
      await cargarLista(cfg.destino);
    } catch (error) {
      reportarError("No se pudo crear el respaldo", error);
    } finally {
      setOcupado(null);
    }
  }, [cfg.destino, passphrase, cargarLista]);

  const probar = useCallback(async (item: InfoRespaldo) => {
    setOcupado(`probar:${item.archivo}`);
    setPrueba(null);
    try {
      const resultado = normalizarPrueba(
        await probarRespaldo(item.archivo, passphrase.trim() || undefined),
      );
      setPrueba(resultado);
      if (!resultado.ok) reportarError("El respaldo no pasó la prueba", resultado.mensaje);
    } catch (error) {
      reportarError("El respaldo no pasó la prueba", error);
    } finally {
      setOcupado(null);
    }
  }, [passphrase]);

  const confirmarRestaurar = useCallback(async () => {
    if (!porRestaurar) return;
    setOcupado("restaurar");
    try {
      await restaurarRespaldo(porRestaurar, passphrase.trim() || undefined);
      notificarExito("Restauración iniciada; la app se reiniciará");
      setPorRestaurar(null);
    } catch (error) {
      // Si la app se reinició, la promesa no resuelve: esto es un fallo real.
      reportarError("No se pudo restaurar el respaldo", error);
    } finally {
      setOcupado(null);
    }
  }, [porRestaurar, passphrase]);

  return {
    cfg, setCfg, lista, passphrase, setPassphrase, prueba, ocupado, cargando,
    porRestaurar, setPorRestaurar, aviso, elegirDestino, guardar, crearAhora,
    probar, confirmarRestaurar,
  };
}
