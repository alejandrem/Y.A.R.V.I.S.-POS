// Formulario para amarrar la cuenta Google del dueño (login OAuth del
// admin) y su Client ID. Autocontenido: carga el perfil, edita los dos
// campos y guarda vía guardar_google_config. La contraseña local no se
// toca: sin internet se entra con clave como siempre.

import { useEffect, useState } from "react";
import { notificarExito } from "../../../../components/notificaciones";
import { reportarError } from "../../../../services/tauri";
import { obtenerAdminData, guardarGoogleConfig } from "../../../../services/auth";

const FormularioGoogle = () => {
  const [email, setEmail] = useState("");
  const [clientId, setClientId] = useState("");
  const [cargando, setCargando] = useState(true);
  const [guardando, setGuardando] = useState(false);

  useEffect(() => {
    obtenerAdminData()
      .then((perfil) => {
        if (perfil) {
          setEmail(perfil.google_email || "");
          setClientId(perfil.google_client_id || "");
        }
      })
      .catch((e) => reportarError("No se pudo cargar tu cuenta Google", e))
      .finally(() => setCargando(false));
  }, []);

  const guardar = async () => {
    setGuardando(true);
    try {
      const msg = await guardarGoogleConfig(email, clientId);
      notificarExito(msg);
    } catch (error) {
      reportarError("No se pudo vincular tu cuenta Google", error);
    } finally {
      setGuardando(false);
    }
  };

  return (
    <div className="bg-white p-8 rounded-[2.5rem] border border-neutral-100 space-y-6 shadow-sm">
      <h3 className="text-[10px] font-black text-neutral-400 uppercase tracking-[0.4em] mb-4">Cuenta Google</h3>

      {cargando ? (
        <p className="text-[10px] font-bold text-neutral-300 uppercase tracking-widest animate-pulse">Cargando…</p>
      ) : (
        <div className="space-y-4">
          <div className="group">
            <label className="text-[9px] font-black text-neutral-400 uppercase ml-2 mb-1 block group-focus-within:text-neutral-900 transition-colors">Correo del dueño</label>
            <input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              className="w-full bg-neutral-50 border border-neutral-100 px-6 py-4 rounded-2xl text-xs font-bold focus:outline-none focus:ring-4 focus:ring-neutral-900/5 focus:border-neutral-900 transition-all"
              placeholder="tucorreo@gmail.com"
            />
          </div>

          <div className="group">
            <label className="text-[9px] font-black text-neutral-400 uppercase ml-2 mb-1 block group-focus-within:text-neutral-900 transition-colors">Client ID (app de escritorio)</label>
            <input
              type="text"
              value={clientId}
              onChange={(e) => setClientId(e.target.value)}
              className="w-full bg-neutral-50 border border-neutral-100 px-6 py-4 rounded-2xl text-xs font-bold focus:outline-none focus:ring-4 focus:ring-neutral-900/5 focus:border-neutral-900 transition-all"
              placeholder="Vacío = usa YARVIS_GOOGLE_CLIENT_ID"
            />
          </div>

          <button
            onClick={guardar}
            disabled={guardando}
            className="w-full bg-neutral-900 text-white py-4 rounded-2xl text-[10px] font-black uppercase tracking-widest hover:scale-[1.02] active:scale-95 transition-all shadow-lg disabled:opacity-40"
          >
            {guardando ? "Vinculando…" : "Vincular cuenta"}
          </button>
        </div>
      )}
    </div>
  );
};

export default FormularioGoogle;
