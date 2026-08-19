// Hook personalizado para cargar, manejar el estado y guardar (persistir) 
// los datos generales del administrador y la tienda (nombres, ubicación, etc).

import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export const PASS_PLACEHOLDER = "\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022";

export function useDatosAdmin(
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

  const showSuccess = useCallback((msg: string) => {
    setSuccessMessage(msg);
    setTimeout(() => setSuccessMessage(""), 3000);
  }, []);

  const handleUpdate = useCallback(async () => {
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
  }, [currentAdminName, currentStoreName, currentPass, passwordChanged, location, cp, showSuccess]);

  // Botón independiente para guardar solo Datos de Identidad (nombre, tienda, ubicación, CP)
  const handleSaveIdentity = useCallback(async () => {
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
  }, [currentAdminName, currentStoreName, location, cp, showSuccess]);

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