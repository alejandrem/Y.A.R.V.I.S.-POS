// ═══════════════════════════════════════════════════════════════════════════
// MODAL ALTA PROVEEDOR — 3 campos veloces + Guardar o encadenar compra.
// Tarea única: capturar nombre (forzoso), teléfono y correo opcionales.
// ═══════════════════════════════════════════════════════════════════════════

import { useState } from "react";
import { reportarError } from "../../../../services/tauri";
import { notificarExito } from "../../../../components/notificaciones";
import { guardarProveedor, type Proveedor } from "../../../../services/proveedores";

interface ModalAltaProps {
  onCerrar: () => void;
  onGuardado: (prov: Proveedor) => void;
  onComprar: (prov: Proveedor) => void;
}

const inputCls =
  "w-full px-5 py-4 bg-neutral-50 border-2 border-neutral-100 rounded-2xl text-base font-black text-neutral-900 placeholder:text-neutral-300 placeholder:font-bold placeholder:text-sm focus:outline-none focus:border-neutral-900 focus:ring-8 focus:ring-neutral-900/5 transition-all";

const ModalAlta = ({ onCerrar, onGuardado, onComprar }: ModalAltaProps) => {
  const [nombre, setNombre] = useState("");
  const [telefono, setTelefono] = useState("");
  const [correo, setCorreo] = useState("");
  const [busy, setBusy] = useState(false);

  const guardar = async (yComprar: boolean) => {
    if (!nombre.trim()) {
      reportarError("Escribe el nombre del proveedor", "Falta el nombre");
      return;
    }
    setBusy(true);
    try {
      const id = await guardarProveedor(nombre, telefono || null, correo || null);
      const prov: Proveedor = {
        id, nombre: nombre.trim(), telefono: telefono.trim() || null,
        correo: correo.trim() || null, total_compras: 0, total_pagado: 0,
      };
      notificarExito(`Proveedor ${prov.nombre} guardado`);
      if (yComprar) onComprar(prov);
      else onGuardado(prov);
    } catch (e) {
      reportarError("No se pudo guardar el proveedor", e);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4 backdrop-blur-sm" onClick={onCerrar}>
      <div
        className="w-full max-w-lg bg-white rounded-[2rem] shadow-2xl p-6 sm:p-8 space-y-5 animate-in fade-in zoom-in-95 duration-200"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="text-center">
          <h3 className="text-2xl font-black text-neutral-900 uppercase tracking-tight">Nuevo proveedor</h3>
          <div className="h-1.5 w-12 bg-neutral-900 rounded-full mx-auto mt-2" />
        </div>

        <div>
          <label className="text-[10px] font-black uppercase tracking-widest text-neutral-400 ml-1">Nombre del proveedor *</label>
          <input
            autoFocus
            value={nombre}
            onChange={(e) => setNombre(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") guardar(false);
            }}
            placeholder="Don Chuy"
            className={`${inputCls} mt-2`}
          />
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div>
            <label className="text-[10px] font-black uppercase tracking-widest text-neutral-400 ml-1">Teléfono (opcional)</label>
            <input value={telefono} onChange={(e) => setTelefono(e.target.value)} placeholder="246 123 4567" className={`${inputCls} mt-2`} />
          </div>
          <div>
            <label className="text-[10px] font-black uppercase tracking-widest text-neutral-400 ml-1">Correo (opcional)</label>
            <input value={correo} onChange={(e) => setCorreo(e.target.value)} placeholder="donchuy@mail.com" className={`${inputCls} mt-2`} />
          </div>
        </div>

        <button
          disabled={busy}
          onClick={() => guardar(false)}
          className="w-full py-4 rounded-2xl border-2 border-neutral-900 text-neutral-900 text-xs font-black uppercase tracking-widest hover:bg-neutral-900 hover:text-white transition-all disabled:opacity-40"
        >
          {busy ? "Guardando..." : "Guardar"}
        </button>
        <button
          disabled={busy}
          onClick={() => guardar(true)}
          className="w-full py-5 rounded-2xl bg-neutral-950 text-white text-xs font-black uppercase tracking-widest shadow-xl shadow-neutral-300 hover:scale-[1.01] active:scale-95 transition-all disabled:opacity-40"
        >
          {busy ? "Guardando..." : "Agregar compra a proveedor →"}
        </button>
        <button onClick={onCerrar} className="w-full py-2 text-[10px] font-black uppercase tracking-widest text-neutral-400 hover:text-neutral-900 transition-colors">
          Cancelar
        </button>
      </div>
    </div>
  );
};

export default ModalAlta;
