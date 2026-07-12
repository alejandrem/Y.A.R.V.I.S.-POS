import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ModalEmpleadosProps {
  onClose: () => void;
  onSaved: () => void;
}

const ModalEmpleados = ({ onClose, onSaved }: ModalEmpleadosProps) => {
  const [name, setName] = useState("");
  const [pass, setPass] = useState("");
  const [confirmPass, setConfirmPass] = useState("");
  const [showPass, setShowPass] = useState(false);

  const handleSave = async () => {
    if (!name.trim()) {
      alert("El nombre es obligatorio");
      return;
    }
    if (pass.length < 6 || !/[A-Za-z]/.test(pass) || !/[0-9]/.test(pass)) {
      alert("La contraseña debe tener al menos 6 caracteres, con letras y números");
      return;
    }
    if (pass !== confirmPass) {
      alert("Las contraseñas no coinciden");
      return;
    }
    try {
      await invoke("guardar_empleado", { name: name.trim(), pass });
      onSaved();
      onClose();
    } catch (error) {
      console.error("Error al guardar empleado:", error);
      alert("Error al guardar empleado");
    }
  };

  return (
    <div
      className="fixed inset-0 bg-black/40 backdrop-blur-sm z-50 flex items-center justify-center"
      onClick={onClose}
    >
      <div
        className="bg-white rounded-[2rem] shadow-2xl w-full max-w-sm p-8 space-y-5 animate-in zoom-in-95 duration-200"
        onClick={(e) => e.stopPropagation()}
      >
        <header className="text-center">
          <h2 className="text-lg font-black text-neutral-900 uppercase">
            Nuevo Registro
          </h2>
          <div className="h-0.5 w-8 bg-neutral-900 mx-auto mt-2 rounded-full"></div>
          <p className="text-[10px] font-bold text-neutral-400 uppercase tracking-widest mt-2">
            Perfil de Acceso
          </p>
        </header>

        <div className="space-y-3">
          <div className="space-y-1">
            <label className="text-[10px] font-bold text-neutral-500 uppercase tracking-wider ml-1">
              Nombre del Empleado
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleSave()}
              placeholder="Ej. Peter Parker"
              className="w-full px-3 py-2 rounded-xl bg-neutral-50 border border-neutral-200 text-sm focus:outline-none focus:border-neutral-900"
            />
          </div>

          <div className="space-y-1">
            <label className="text-[10px] font-bold text-neutral-500 uppercase tracking-wider ml-1">
              Crear Contraseña
            </label>
            <div className="relative">
              <input
                type={showPass ? "text" : "password"}
                value={pass}
                onChange={(e) => setPass(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleSave()}
                className="w-full px-3 py-2 pr-10 rounded-xl bg-neutral-50 border border-neutral-200 text-sm focus:outline-none focus:border-neutral-900"
              />
              <button
                onClick={() => setShowPass(!showPass)}
                className="absolute right-2 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-neutral-600 transition-colors"
              >
                {showPass ? "👁️" : "🙈"}
              </button>
            </div>
          </div>

          <div className="space-y-1">
            <label className="text-[10px] font-bold text-neutral-500 uppercase tracking-wider ml-1">
              Confirmar Contraseña
            </label>
            <input
              type="password"
              value={confirmPass}
              onChange={(e) => setConfirmPass(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleSave()}
              className="w-full px-3 py-2 rounded-xl bg-neutral-50 border border-neutral-200 text-sm focus:outline-none focus:border-neutral-900"
            />
          </div>
        </div>

        <div className="pt-2 space-y-2">
          <button
            onClick={handleSave}
            className="w-full py-4 rounded-xl bg-neutral-900 text-white text-xs font-black uppercase tracking-[0.2em] hover:bg-neutral-800 transition-all shadow-xl shadow-neutral-200"
          >
            Guardar Usuario
          </button>
          <button
            onClick={onClose}
            className="w-full py-3 text-[10px] font-bold text-neutral-400 uppercase tracking-widest"
          >
            Cancelar
          </button>
        </div>
      </div>
    </div>
  );
};

export default ModalEmpleados;
