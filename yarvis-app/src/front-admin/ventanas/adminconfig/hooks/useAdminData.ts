import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export const PASS_PLACEHOLDER = "\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022";

export function useAdminData(
  adminName: string,
  storeName: string,
  initialLocation = "",
  initialCp = ""
) {
  const [currentAdminName, setCurrentAdminName] = useState(adminName);
  const [currentStoreName, setCurrentStoreName] = useState(storeName);
  const [currentPass, setCurrentPass] = useState(PASS_PLACEHOLDER);
  const [passwordChanged, setPasswordChanged] = useState(false);
  const [location, setLocation] = useState(initialLocation);
  const [cp, setCp] = useState(initialCp);
  const [successMessage, setSuccessMessage] = useState("");

  const showSuccess = (msg: string) => {
    setSuccessMessage(msg);
    setTimeout(() => setSuccessMessage(""), 3000);
  };

  const handleUpdate = async () => {
    try {
      // Solo mandar contraseña si el usuario la cambió y no está vacía
      const passToSend = passwordChanged && currentPass.trim() !== "" ? currentPass : "";
      await invoke("update_admin_data", {
        nombre: currentAdminName,
        tienda: currentStoreName,
        pass: passToSend,
        ubicacion: location,
        cp: cp
      });
      // Resetear feedback visual de contraseña
      if (passwordChanged) {
        setCurrentPass(PASS_PLACEHOLDER);
        setPasswordChanged(false);
      }
      showSuccess("Contraseña actualizada exitosamente");
    } catch (error) {
      console.error("Error al actualizar:", error);
      alert("Hubo una falla al guardar los datos.");
    }
  };

  // Botón independiente para guardar solo Datos de Identidad (nombre, tienda, ubicación, CP)
  const handleSaveIdentity = async () => {
    try {
      await invoke("update_admin_data", {
        nombre: currentAdminName,
        tienda: currentStoreName,
        pass: "",  // vacío → Rust no re-hashea
        ubicacion: location,
        cp: cp
      });
      showSuccess("Cambios guardados exitosamente");
    } catch (error) {
      console.error("Error al guardar datos de identidad:", error);
      alert("Hubo una falla al guardar los datos.");
    }
  };

  return {
    currentAdminName,
    setCurrentAdminName,
    currentStoreName,
    setCurrentStoreName,
    currentPass,
    setCurrentPass,
    passwordChanged,
    setPasswordChanged,
    location,
    setLocation,
    cp,
    setCp,
    successMessage,
    handleUpdate,
    handleSaveIdentity,
  };
}