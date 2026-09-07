// TEST — Contrato post keep-alive (commit 33f7218 intacto en espíritu).
//
// El keep-alive con `hidden` se revirtió por mala práctica (árboles ocultos
// en el DOM: RAM, renders inútiles, a11y). El contrato que estos tests
// blindan es el que importa:
//
//   1. Solo la pestaña activa está en el DOM (nada oculto simultáneo).
//   2. Lo que debe sobrevivir vive en providers: el progreso del lote, el
//      stream del chat y el carrito sobreviven al desmontar la pantalla.
//      La UI nunca miente sobre lo que el backend sigue haciendo.

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, act } from "@testing-library/react";
import { mockInvoke, mockListen } from "./setup";
import AdminDashboard from "../front-admin/AdminDashboard";
import { CartProvider, useCart } from "../front-empleado/ventanas/emplea_new_venta/CartProvider";
import { ChatProvider, useChat } from "../front-admin/ventanas/adminyarvis/ChatProvider";

const baseProps = {
  setActiveTab: () => {},
  onLogout: () => {},
  adminName: "Admin",
  storeName: "Tienda",
  adminPass: "x",
};

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockResolvedValue(undefined);
  mockListen.mockClear();
  localStorage.removeItem("yarvis_chat_sessions_chat-keepalive");
});

describe("dashboard · sin árboles ocultos", () => {
  it("la pestaña inactiva NO queda en el DOM (ni siquiera oculta)", () => {
    const { rerender } = render(<AdminDashboard activeTab="ventas" {...baseProps} />);
    expect(screen.getByText("El pulso de tu tienda, aprendido de tus tickets")).toBeInTheDocument();

    rerender(<AdminDashboard activeTab="parseador" {...baseProps} />);
    expect(screen.getByText("Parseador de Tickets")).toBeInTheDocument();
    // Sin `hidden`, sin display:none: desmontada de verdad.
    expect(screen.queryByText("El pulso de tu tienda, aprendido de tus tickets")).toBeNull();
    expect(document.querySelectorAll("div.hidden").length).toBe(0);

    rerender(<AdminDashboard activeTab="ventas" {...baseProps} />);
    expect(screen.getByText("El pulso de tu tienda, aprendido de tus tickets")).toBeInTheDocument();
    expect(screen.queryByText("Parseador de Tickets")).toBeNull();
  });
});

describe("cart · sobrevive al cambio de pestaña", () => {
  const Probe = () => {
    const c = useCart();
    return (
      <>
        <button
          onClick={() =>
            c.addToCart({ id: 1, nombre: "COCA", precio_venta: 18, stock: 50 } as never)
          }
        >
          agregar
        </button>
        <span data-testid="n">{c.cart.length}</span>
        <span data-testid="total">{c.cartTotal}</span>
      </>
    );
  };

  it("el carrito a medias sigue ahí al volver del otro módulo", () => {
    const tree = (
      <CartProvider>
        <Probe />
      </CartProvider>
    );
    const { rerender } = render(tree);
    act(() => {
      screen.getByText("agregar").click();
    });
    expect(screen.getByTestId("n").textContent).toBe("1");
    expect(screen.getByTestId("total").textContent).toBe("18");

    // "Cambia de pestaña": el consumidor se desmonta, el provider no.
    rerender(
      <CartProvider>
        <></>
      </CartProvider>,
    );
    rerender(tree);
    expect(screen.getByTestId("n").textContent).toBe("1");
    expect(screen.getByTestId("total").textContent).toBe("18");
  });
});

describe("chat · el stream sobrevive al cambio de pestaña", () => {
  const Probe = () => {
    const chat = useChat();
    return (
      <>
        <button onClick={() => chat.stream.handleSend("hola")}>enviar</button>
        <span data-testid="streaming">{chat.stream.streamingText}</span>
        <span data-testid="nmsgs">{chat.messages.length}</span>
      </>
    );
  };
  const tree = (
    <ChatProvider role="admin" userId="chat-keepalive">
      <Probe />
    </ChatProvider>
  );

  const handlerFor = (event: string) => {
    const call = mockListen.mock.calls.find((c) => c[0] === event);
    if (!call) throw new Error(`sin listener para ${event}`);
    return call[1] as (e: { payload: never }) => void;
  };

  it("los tokens y la respuesta llegan aunque la pantalla se desmonte en medio", async () => {
    const { rerender } = render(tree);
    await act(async () => {
      screen.getByText("enviar").click();
    });
    // El hook registra sus 5 listeners al enviar.
    expect(mockListen).toHaveBeenCalledWith("chat-token", expect.anything());
    expect(mockInvoke).toHaveBeenCalledWith(
      "send_chat_stream",
      expect.objectContaining({ role: "admin" }),
    );

    // Llegan tokens mientras el usuario... se va a otro módulo.
    await act(async () => {
      handlerFor("chat-token")({ payload: { token: "mundo", model: "m" } as never });
    });
    expect(screen.getByTestId("streaming").textContent).toBe("mundo");

    // Desmonta y remonta la pantalla: el stream sigue vivo en el provider.
    rerender(
      <ChatProvider role="admin" userId="chat-keepalive">
        <></>
      </ChatProvider>,
    );
    rerender(tree);
    expect(screen.getByTestId("streaming").textContent).toBe("mundo");

    // El backend termina: la respuesta cae en la sesión aunque nadie la mire.
    await act(async () => {
      handlerFor("chat-complete")({ payload: { response: "hola mundo", model: "m" } as never });
    });
    // user + assistant.
    expect(screen.getByTestId("nmsgs").textContent).toBe("2");
    expect(screen.getByTestId("streaming").textContent).toBe("");
  });
});
