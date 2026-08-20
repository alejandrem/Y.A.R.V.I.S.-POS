// Alta de empleado.
// Piel reconstruida con ModalShell + campos gorditos + morphicon de contraseña.
// La lógica de validación y guardado se conserva intacta.
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MorphIcon } from "morphicons/react";
import { ModalShell, Campo, inputCls, ICONO_USUARIO, ICONO_OJO, ICONO_OJO_OCULTO, ICONO_CHECK } from "../../../components/ui";

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
    <ModalShell icono={ICONO_USUARIO} titulo="Nuevo Registro" subtitulo="Perfil de Acceso" onClose={onClose}>
      <div className="space-y-4">
        <Campo label="Nombre del Empleado">
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleSave()}
            placeholder="Ej. Peter Parker"
            autoFocus
            className={inputCls}
          />
        </Campo>

        <Campo label="Crear Contraseña">
          <div className="relative">
            <input
              type={showPass ? "text" : "password"}
              value={pass}
              onChange={(e) => setPass(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleSave()}
              placeholder="••••••••"
              className={`${inputCls} pr-12`}
            />
            <button
              type="button"
              onClick={() => setShowPass(!showPass)}
              aria-label={showPass ? "Ocultar contraseña" : "Mostrar contraseña"}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-neutral-950 transition-colors"
            >
              <MorphIcon icon={showPass ? ICONO_OJO_OCULTO : ICONO_OJO} size={17} strokeWidth={2} spring="snappy" reducedMotion="user" />
            </button>
          </div>
        </Campo>

        <Campo label="Confirmar Contraseña">
          <input
            type={showPass ? "text" : "password"}
            value={confirmPass}
            onChange={(e) => setConfirmPass(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleSave()}
            placeholder="••••••••"
            className={inputCls}
          />
        </Campo>
      </div>

      <div className="pt-2 space-y-2">
        <button
          onClick={handleSave}
          className="w-full inline-flex items-center justify-center gap-2.5 py-4 rounded-xl bg-neutral-950 text-neutral-50 text-xs font-black uppercase tracking-[0.2em] hover:bg-neutral-800 transition-all shadow-xl shadow-neutral-200 active:scale-[0.98]"
        >
          <MorphIcon icon={ICONO_CHECK} size={16} strokeWidth={2.5} spring="snappy" />
          Guardar Usuario
        </button>
        <button
          onClick={onClose}
          className="w-full py-3 text-[10px] font-black text-neutral-400 uppercase tracking-widest hover:text-neutral-900 transition-colors"
        >
          Cancelar
        </button>
      </div>
    </ModalShell>
  );
};

export default ModalEmpleados;