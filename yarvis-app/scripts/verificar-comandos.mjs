// ═══════════════════════════════════════════════════════════════════════════
// verificar-comandos.mjs — Paridad backend <-> frontend de comandos Tauri.
//
// Uso:  npm run verificar:comandos   (sale 0 si todo cuadra, 1 si hay drift)
// Lee:
//   backend: src-tauri/src/lib.rs (bloque generate_handler!, fuente de verdad)
//   frontend: src/**/*.{ts,tsx} (invokeTauri("x") / invoke("x"), excluye test/)
// Reporta:
//   1. duplicados en backend (rompería el match del handler)
//   2. comandos de frontend que NO existen en backend (bug: invoke muerto)
//   3. comandos de backend sin uso en frontend ni tests (revisar: muerto o
//      usado por nombre dinámico). No falla por estos, solo avisa.
// ═══════════════════════════════════════════════════════════════════════════
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const raiz = join(dirname(fileURLToPath(import.meta.url)), "..");
const libRs = join(raiz, "src-tauri", "src", "lib.rs");
const srcDir = join(raiz, "src");

const src = readFileSync(libRs, "utf8");
const ini = src.indexOf("generate_handler![");
if (ini === -1) {
  console.error("[comandos] no se encontró generate_handler![ en lib.rs");
  process.exit(1);
}
const fin = src.indexOf("])", ini);
const bloque = src.slice(ini, fin);

const backend = [];
for (const linea of bloque.split("\n").slice(1)) {
  const t = linea.trim();
  if (!t || t.startsWith("//")) continue;
  const m = t.replace(/,$/, "").match(/::([A-Za-z0-9_]+)$|^([a-z][a-z0-9_]*)$/);
  if (m) backend.push(m[1] ?? m[2]);
}

const dupes = backend.filter((n, i) => backend.indexOf(n) !== i);
let fallos = 0;
if (dupes.length > 0) {
  console.error(`[comandos] DUPLICADOS en backend: ${[...new Set(dupes)].join(", ")}`);
  fallos = 1;
}

const exts = new Set([".ts", ".tsx"]);
const invoca = new Set();
const invocaTests = new Set();
const recoger = (dir, destino) => {
  for (const e of readdirSync(dir)) {
    const p = join(dir, e);
    const st = statSync(p);
    if (st.isDirectory()) {
      recoger(p, p.includes("test") ? invocaTests : destino);
      continue;
    }
    if (![...exts].some((x) => p.endsWith(x))) continue;
    const txt = readFileSync(p, "utf8");
    const re = /(?:invokeTauri|invoke)\s*(?:<[^()]*>)?\s*\(\s*["']([A-Za-z0-9_]+)["']/g;
    let m;
    while ((m = re.exec(txt))) destino.add(m[1]);
    // En tests el invoke está mockeado: el nombre viaja en el expect.
    if (destino === invocaTests) {
      const re2 = /toHaveBeenCalledWith\(\s*["']([A-Za-z0-9_]+)["']/g;
      while ((m = re2.exec(txt))) destino.add(m[1]);
    }
  }
};
recoger(srcDir, invoca);

const setBackend = new Set(backend);
const fantasmas = [...invoca].filter((n) => !setBackend.has(n));
if (fantasmas.length > 0) {
  console.error(`[comandos] FRONTEND invoca comandos que NO existen en backend: ${fantasmas.join(", ")}`);
  fallos = 1;
}

const sinUso = backend.filter((n) => !invoca.has(n) && !invocaTests.has(n));
console.log(`[comandos] backend=${backend.length} frontend(src)=${invoca.size} tests_extra=${[...invocaTests].filter((n) => !invoca.has(n)).length}`);
if (sinUso.length > 0) {
  console.log(`[comandos] AVISO: sin invoke en src ni tests (revisar si muertos o dinámicos): ${sinUso.join(", ")}`);
}
console.log(fallos === 0 ? "[comandos] OK: paridad en verde." : "[comandos] FALLO: hay drift arriba.");
process.exit(fallos);
