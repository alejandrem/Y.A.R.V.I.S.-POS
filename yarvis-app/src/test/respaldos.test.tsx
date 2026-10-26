// ═══════════════════════════════════════════════════════════════════════════
// TESTS ADVERSARIOS — Respaldos.
// Intentan romper la UI con respuestas basura, activar respaldos sin
// carpeta/contraseña y restaurar sin confirmar. Si algo de esto pasa, el
// dueño puede perder la caja o creer que tiene un respaldo que no existe.
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { mockInvoke } from "./setup";
import FormularioRespaldos, {
  formatearBytes,
} from "../front-admin/ventanas/adminconfig/components/FormularioRespaldos";
import {
  normalizarConfig,
  normalizarLista,
  normalizarPrueba,
} from "../front-admin/ventanas/adminconfig/hooks/useRespaldos";
import {
  crearRespaldoAhora,
  guardarConfigRespaldo,
  listarRespaldos,
  probarRespaldo,
  restaurarRespaldo,
} from "../services/respaldos";

const ITEM = {
  archivo: "C:\\Respaldos\\yarvis-20260101-120000.yarvisbak",
  nombre: "yarvis-20260101-120000.yarvisbak",
  bytes: 2048,
  fecha: "2026-01-01 12:00:00",
};

const CONFIG = {
  activo: false,
  destino: "C:\\Respaldos",
  cada_horas: 24,
  retener: 30,
  ultimo_exito: null,
  tiene_passphrase: true,
};

function responder(extra: (cmd: string, args?: unknown) => unknown = () => undefined) {
  mockInvoke.mockImplementation((cmd: string, args?: unknown) => {
    const propio = extra(cmd, args);
    if (propio !== undefined) return Promise.resolve(propio);
    if (cmd === "respaldo_config_leer") return Promise.resolve(CONFIG);
    if (cmd === "respaldo_listar") return Promise.resolve([ITEM]);
    return Promise.resolve(undefined);
  });
}

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockResolvedValue(undefined);
});

describe("respaldos · normalización defensiva", () => {
  it("un mock viejo que devuelve 'ok' no rompe la configuración", () => {
    const cfg = normalizarConfig("ok");
    expect(cfg.activo).toBe(false);
    expect(cfg.destino).toBeNull();
    expect(cfg.cada_horas).toBe(24);
    expect(cfg.retener).toBe(30);
  });

  it("recorta intervalos y retención fuera de rango", () => {
    const cfg = normalizarConfig({
      activo: true,
      destino: "   ",
      cada_horas: -5,
      retener: 9999,
      tiene_passphrase: 1,
    });
    expect(cfg.activo).toBe(true);
    expect(cfg.destino).toBeNull();
    expect(cfg.cada_horas).toBe(1);
    expect(cfg.retener).toBe(100);
    expect(cfg.tiene_passphrase).toBe(false);
  });

  it("la lista ignora basura, rutas vacías y tipos incorrectos", () => {
    expect(normalizarLista("ok")).toEqual([]);
    expect(normalizarLista(null)).toEqual([]);
    expect(normalizarLista([
      null, 42, {}, { archivo: "" }, { archivo: "/x.yarvisbak", bytes: -1 },
    ])).toEqual([
      { archivo: "/x.yarvisbak", nombre: "/x.yarvisbak", bytes: 0, fecha: "" },
    ]);
  });

  it("una prueba inválida nunca se reporta como exitosa", () => {
    const prueba = normalizarPrueba("ok");
    expect(prueba.ok).toBe(false);
    expect(prueba.mensaje.length).toBeGreaterThan(0);
  });

  it("formatea tamaños sin NaN ni negativos", () => {
    expect(formatearBytes(0)).toBe("0 B");
    expect(formatearBytes(-5)).toBe("0 B");
    expect(formatearBytes(Number.NaN)).toBe("0 B");
    expect(formatearBytes(1024)).toBe("1 KB");
    expect(formatearBytes(2048)).toBe("2 KB");
    expect(formatearBytes(5 * 1024 * 1024)).toBe("5 MB");
  });
});

describe("respaldos · servicio Tauri", () => {
  it("usa los nombres y argumentos exactos del backend", async () => {
    await listarRespaldos();
    await guardarConfigRespaldo({
      activo: true, destino: "C:\\USB", cada_horas: 12, retener: 10, passphrase: "clave-larga",
    });
    await crearRespaldoAhora();
    await crearRespaldoAhora("clave-larga");
    await probarRespaldo(ITEM.archivo);
    await restaurarRespaldo(ITEM.archivo, "clave-larga");

    expect(mockInvoke).toHaveBeenCalledWith("respaldo_listar");
    expect(mockInvoke).toHaveBeenCalledWith("respaldo_config_guardar", {
      entrada: {
        activo: true, destino: "C:\\USB", cada_horas: 12, retener: 10, passphrase: "clave-larga",
      },
    });
    expect(mockInvoke).toHaveBeenCalledWith("respaldo_crear_ahora");
    expect(mockInvoke).toHaveBeenCalledWith("respaldo_crear_ahora", { passphrase: "clave-larga" });
    expect(mockInvoke).toHaveBeenCalledWith("respaldo_probar", {
      archivo: ITEM.archivo, passphrase: null,
    });
    expect(mockInvoke).toHaveBeenCalledWith("respaldo_restaurar", {
      archivo: ITEM.archivo, passphrase: "clave-larga",
    });
  });
});

describe("respaldos · UI", () => {
  it("muestra la lista cuando ya hay carpeta configurada", async () => {
    responder();
    render(<FormularioRespaldos />);

    await screen.findByText(ITEM.nombre);
    expect(screen.getByText(/2 KB/)).toBeInTheDocument();
    expect(screen.getByLabelText("Carpeta de respaldos")).toHaveValue("C:\\Respaldos");
  });

  it("no deja activar sin carpeta", async () => {
    responder((cmd) => cmd === "respaldo_config_leer" ? { ...CONFIG, destino: null } : undefined);
    render(<FormularioRespaldos />);

    await screen.findByText("APAGADO");
    fireEvent.click(screen.getByRole("checkbox"));
    fireEvent.click(screen.getByRole("button", { name: /guardar configuración/i }));

    const aviso = await screen.findByRole("alert");
    expect(aviso).toHaveTextContent(/elige una carpeta o usb/i);
    expect(mockInvoke).not.toHaveBeenCalledWith("respaldo_config_guardar", expect.anything());
  });

  it("no deja activar sin contraseña cuando no hay una guardada", async () => {
    responder((cmd) => cmd === "respaldo_config_leer" ? { ...CONFIG, tiene_passphrase: false } : undefined);
    render(<FormularioRespaldos />);
    await screen.findByText(ITEM.nombre);

    fireEvent.click(screen.getByRole("checkbox"));
    fireEvent.click(screen.getByRole("button", { name: /guardar configuración/i }));

    expect(await screen.findByRole("alert")).toHaveTextContent(/contraseña de al menos 8/i);
    expect(mockInvoke).not.toHaveBeenCalledWith("respaldo_config_guardar", expect.anything());
  });

  it("restaurar exige una segunda confirmación y permite cancelar", async () => {
    responder();
    render(<FormularioRespaldos />);
    await screen.findByText(ITEM.nombre);

    fireEvent.click(screen.getByRole("button", { name: /^restaurar$/i }));
    expect(screen.getByText(/reemplaza la base actual/i)).toBeInTheDocument();
    expect(mockInvoke).not.toHaveBeenCalledWith("respaldo_restaurar", expect.anything());

    fireEvent.click(screen.getByRole("button", { name: /cancelar/i }));
    await waitFor(() => {
      expect(screen.queryByText(/reemplaza la base actual/i)).not.toBeInTheDocument();
    });
    expect(mockInvoke).not.toHaveBeenCalledWith("respaldo_restaurar", expect.anything());
  });

  it("probar un respaldo muestra el conteo real de la base", async () => {
    responder((cmd) => cmd === "respaldo_probar" ? {
      ok: true, mensaje: "Respaldo válido: 3 ventas, 2 productos y 1 usuarios.",
      tablas: 20, usuarios: 1, ventas: 3, productos: 2, detalle_ventas: 4, version_esquema: 18,
    } : undefined);
    render(<FormularioRespaldos />);
    await screen.findByText(ITEM.nombre);

    fireEvent.click(screen.getByRole("button", { name: /^probar$/i }));
    expect(await screen.findByText(/Respaldo válido/)).toBeInTheDocument();
    expect(screen.getAllByText(/3 ventas/).length).toBeGreaterThan(0);
    expect(screen.getByText(/esquema v18/)).toBeInTheDocument();
  });

  it("una respuesta basura no tumba Configuración", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "respaldo_config_leer") return Promise.resolve("ok");
      if (cmd === "respaldo_listar") return Promise.reject(new Error("USB desconectado"));
      return Promise.resolve(undefined);
    });
    render(<FormularioRespaldos />);

    expect(await screen.findByText("APAGADO")).toBeInTheDocument();
    expect(screen.getByLabelText("Carpeta de respaldos")).toHaveAttribute(
      "placeholder",
      "Sin carpeta elegida",
    );
    expect(screen.queryByText(ITEM.nombre)).not.toBeInTheDocument();
  });
});
