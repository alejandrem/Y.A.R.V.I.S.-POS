// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Módulo PARSEADOR (parseador de tickets).
// Verifica que el panel de importación renderiza sin crash y que no toca
// al backend hasta que el usuario actúa.
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, beforeEach } from "vitest";
import { render } from "@testing-library/react";
import { mockInvoke } from "./setup";
import Parseador from "../front-admin/ventanas/parseador/parseador";

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockResolvedValue(undefined);
});

describe("parseador · panel de importación", () => {
  it("renderiza el panel sin crash y sin invocar al backend", () => {
    const { container } = render(<Parseador />);
    expect(container).toBeTruthy();
    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
