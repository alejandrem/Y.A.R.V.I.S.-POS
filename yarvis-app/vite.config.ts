import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },

  // 4. code-splitting: el bundle monolítico de 1.25MB tarda en abrir en
  //    laptops viejas. Se separan los vendors pesados (recharts, markdown)
  //    para que el navegador cachee y el chunk inicial quede liviano.
  //    Los paneles ya van con React.lazy desde App.tsx.
  build: {
    chunkSizeWarningLimit: 500,
    rollupOptions: {
      output: {
        manualChunks: {
          // OJO: no se separa "react": quedaría un chunk vacío porque el
          // entry inicial lo necesita sí o sí. Solo vendors bajo demanda.
          "vendor-charts": ["recharts"],
          "vendor-markdown": ["react-markdown", "remark-gfm"],
        },
      },
    },
  },
}));
