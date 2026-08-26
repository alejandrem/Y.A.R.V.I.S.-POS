# Comandos de Desarrollo

## Arranque rapido

```bash
./run.sh        # Linux: verifica npm + cargo, instala deps si faltan, corre npm run tauri dev
.\run.bat       # Windows
./reset.sh      # Borra yarvis.db, -shm, -wal y caches de app_data_dir (conserva codigo, modelos y .git)
```

## Build de produccion

El build directo con `npm run tauri build` omite variables necesarias para el motor llama.cpp. Usa el wrapper:

```bash
./yarvis-app/build.sh   # NO_STRIP=1 + LD_LIBRARY_PATH para libllama.so.0 -> binario + .deb + .rpm + .AppImage
# Artefactos en yarvis-app/src-tauri/target/release/bundle/
```

Requisitos del bundle AppImage en Arch Linux: `fuse2` instalado y `/usr/lib/gdk-pixbuf-2.0/2.10.0/loaders` disponible. Ver bugs-resueltos.md Bug B1.

## Si falla la compilacion de Rust (toolchain)

Sucede cuando el toolchain por defecto no es stable o quedo un cache de una compilacion parcial.

Windows:

```
rustup update stable
rustup default stable
cd yarvis-app/src-tauri
cargo clean
cd ../..
.\run.bat
```

Linux:

```
rustup update stable
rustup default stable
cd yarvis-app/src-tauri
cargo clean
cd ../..
./run.sh
```

## Tests y verificacion

```bash
# Backend Rust + motor IA (97 comandos + predicciones + parseador)
cargo test --manifest-path yarvis-app/src-tauri/Cargo.toml
cargo test --manifest-path src-ia/Cargo.toml

# Frontend
npm --prefix yarvis-app test        # vitest run
npm --prefix yarvis-app run build   # tsc + vite build (verificacion de tipos)
```

## Variables de entorno utiles

- RUST_LOG=info|debug — nivel de tracing del backend (lib.rs:20). Por defecto info.
- APPIMAGE_EXTRACT_AND_RUN=1 — evita necesitar fuse2 montando el AppImage por extraccion (alternativa a instalar fuse2).

## Estructura de scripts

- run.sh / run.bat: orquestadores de dev. No tocan la DB.
- reset.sh: limpieza destructiva solo de datos de app_data_dir (yarvis.db, caches). No borra src-ia/target ni node_modules.
- yarvis-app/build.sh: unico punto de entrada para release. No usar npm run tauri build directo en produccion.
