---
name: Búsqueda de Código
description: Guía para buscar en el código del proyecto de forma eficiente usando grep (contenido) y glob (nombres de archivo), evitando lecturas completas de archivos grandes. La relevancia de esta 
---

# Búsqueda de Código

Cuando necesites encontrar algo en este proyecto, sigue estas reglas:

## Estrategia

1. **Primero ubica, después lee.** Nunca leas archivos completos para "buscar algo". Usa `grep` para contenido y `glob` para nombres de archivo/rutas.
2. **Afina antes de ampliar.** Si una búsqueda devuelve demasiados resultados, añade contexto (ruta, tipo de archivo, patrón más específico) en vez de leer todo.
3. **Archivo conocido → leer directo.** Si ya sabes el archivo y la línea aproximada, `read` con `offset`/`limit` es más barato que otra búsqueda.

## Búsqueda de contenido (grep)

| Quiero encontrar | Ejemplo |
| --- | --- |
| Un símbolo (función, clase, variable) | `pattern: "function parseConfig"`, `include: "*.ts"` |
| Todas las llamadas a un método | `pattern: "\.connect\("` |
| Texto literal con caracteres raros | usar `literal: true` |
| Buscar sin distinguir mayúsculas | `caseSensitive: false` |
| Solo en una carpeta | `path: "src/services"` |
| Solo ciertos archivos | `include: "*.{tsx,vue}"` |

- Empieza con búsquedas **case-sensitive y acotadas**; sube a `caseSensitive: false` o rutas más amplias solo si no hay resultados.
- Si el patrón usa `**` para rutas, recuerda que `glob` es la herramienta correcta, no `grep`.

## Búsqueda de archivos (glob)

- `pattern: "src/**/*.test.ts"` — todos los tests bajo `src`.
- `pattern: "**/*.md"` — toda la documentación.
- Combina `path` para acotar a un subdirectorio y `limit` si esperas muchos resultados.

## Errores comunes

- **Grep sobre archivos enormes:** acota con `path`/`include` antes de lanzar el patrón.
- **Buscar con regex cuando es texto fijo:** usa `literal: true` para evitar errores de escaping.
- **Asumir la ubicación:** no supongas dónde está un componente; búscalo, aunque "recuerdes" haberlo visto.
- **Abrir `node_modules`/artefactos generados:** acota las búsquedas al código fuente (`src/`, `app/`, etc.).

## Resumen rápido

- ¿Dónde está esto? → `glob`
- ¿Qué archivos lo usan/mencionan? → `grep`
- ¿Qué hace este código? → `grep` primero, `read` después, solo lo necesario.
