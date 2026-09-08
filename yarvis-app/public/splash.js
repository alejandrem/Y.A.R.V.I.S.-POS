/* ═══════════════════════════════════════════════════════════════════
   SPLASH YARVIS — Corre ANTES del primer paint (script clásico en <head>).
   1. Pre-aplica la clase `dark` según `yarvis-theme` (evita el flash de
      tema: React la aplicaría recién en su primer efecto).
   2. Expone `window.__yarvisHideSplash()` para que App la oculte al estar
      lista, con failsafe de 12s por si el bundle nunca arranca.
   Sin dependencias. Compatible con la CSP (archivo propio, script-src self).
   ═══════════════════════════════════════════════════════════════════ */
(function () {
  // Cuánto TIEMPO MÍNIMO luce la animación aunque React ya esté listo.
  // Súbelo o bájalo a gusto: es el único número que controla el show.
  var MIN_VISIBLE_MS = 1500;
  var mostradoEn = Date.now();

  try {
    var stored = null;
    try {
      stored = window.localStorage.getItem("yarvis-theme");
    } catch (e) {
      stored = null;
    }
    var dark =
      stored === "oscuro" ||
      (stored !== "claro" &&
        window.matchMedia &&
        window.matchMedia("(prefers-color-scheme: dark)").matches);
    if (dark) {
      document.documentElement.classList.add("dark");
    }

    window.__yarvisHideSplash = function () {
      var splash = document.getElementById("yarvis-splash");
      if (!splash || splash.classList.contains("done")) return;
      // Si React estuvo listo antes del mínimo, se espera lo que falte:
      // la animación siempre se ve completa al menos una vez.
      var espera = MIN_VISIBLE_MS - (Date.now() - mostradoEn);
      if (espera < 0) espera = 0;
      window.setTimeout(function () {
        splash.classList.add("done");
        window.setTimeout(function () {
          if (splash.parentNode) splash.parentNode.removeChild(splash);
        }, 450);
      }, espera);
    };

    // Failsafe: si React no bootea, no dejar la pantalla colgada.
    window.setTimeout(function () {
      if (window.__yarvisHideSplash) window.__yarvisHideSplash();
    }, 12000);
  } catch (e) {
    /* splash decorativo: jamás debe romper el arranque */
  }
})();
