// ═══════════════════════════════════════════════════════════════════════════
// AJUSTES DEL EMPLEADO — Pantalla de configuración personal.
// Tarea única: orquestar las secciones (datos de sesión, apariencia) y
// cargar el contexto desde el backend. El contenido crecerá por secciones;
// debajo de Apariencia queda espacio deliberadamente libre.
// ═══════════════════════════════════════════════════════════════════════════

import { useState, useEffect, lazy, Suspense } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTheme } from "../../../hooks/useTheme";
import { notificarError } from "../../../components/notificaciones";
import PastillaTema from "./componentes/pastilla-tema";
import DatosSesion from "./componentes/datos-sesion";

const Libro = lazy(() => import("./datos inutiles/Libro"));

const ajustesNav = {
  id: "ajustes",
  label: "AJUSTES",
  icon: (
    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
      <circle cx="12" cy="12" r="3" />
    </svg>
  ),
};

/** Forma del comando get_tienda_info (backend: models::TiendaInfo). */
interface TiendaInfoBackend {
  nombre: string | null;
  ubicacion: string | null;
  cp: string | null;
}

interface AjustesProps {
  operatorName?: string;
}

function Ajustes({ operatorName = "" }: AjustesProps) {
  const { theme, setTheme } = useTheme();
  const [tiendaInfo, setTiendaInfo] = useState<TiendaInfoBackend | null>(null);

  useEffect(() => {
    invoke<TiendaInfoBackend>("get_tienda_info")
      .then(setTiendaInfo)
      .catch((e) => {
        console.error("[AJUSTES] no se pudo cargar la información de la tienda:", e);
        notificarError("No se pudo cargar la información de la tienda", e);
      });
  }, []);

  return (
    <div className="w-full max-w-[1200px] mx-auto space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-500">
      {/* HEADER */}
      <header>
        <h2 className="text-3xl font-black text-neutral-900 uppercase tracking-tight">Ajustes</h2>
        <p className="text-[10px] font-black text-neutral-400 uppercase tracking-[0.3em] mt-1">
          Preferencias de tu cuenta en este punto de venta
        </p>
      </header>

      <div className="grid grid-cols-2 gap-5 items-start">
        <DatosSesion
          nombreEmpleado={operatorName}
          tienda={tiendaInfo?.nombre ?? null}
          ubicacion={tiendaInfo?.ubicacion ?? null}
          cp={tiendaInfo?.cp ?? null}
        />

        {/* APARIENCIA */}
        <section className="bg-white rounded-[2.5rem] border border-neutral-200 shadow-sm p-6 sm:p-8">
          <p className="text-[10px] font-black text-neutral-400 uppercase tracking-widest mb-1">
            Apariencia
          </p>
          <p className="text-[11px] font-bold text-neutral-500 mb-5">
            Elige cómo se ve Y.A.R.V.I.S. "Sistema" sigue la preferencia de tu equipo.
          </p>
          <PastillaTema tema={theme} onCambiar={setTheme} />
        </section>
      </div>

      {/* LIBRO - DATOS INUTILES (lazy: no infla el bundle de venta) */}
      <Suspense
        fallback={
          <div className="w-full max-w-[1200px] h-[620px] bg-white border-2 border-neutral-200 rounded-[1.8rem] animate-pulse flex items-center justify-center">
            <span className="font-mono text-[11px] font-black tracking-widest text-neutral-400">
              CARGANDO MANUAL...
            </span>
          </div>
        }
      >
        <Libro />
      </Suspense>

      {/* Espacio reservado para futuras secciones */}
    </div>
  );
}

export default Ajustes;
export { ajustesNav };
