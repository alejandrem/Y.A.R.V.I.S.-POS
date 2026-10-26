# Skill: Crear Issues en GitHub con gh CLI

## Resumen
Esta skill documenta cómo crear issues en GitHub usando `gh CLI` correctamente, evitando problemas de escaping de comillas y caracteres especiales en PowerShell.

---

## Requisitos previos

```powershell
# gh debe estar en PATH (reiniciar terminal tras instalar)
$env:Path += ";C:\Program Files\GitHub CLI"
gh --version  # Verificar
gh auth login  # Autenticar una sola vez
```

---

## Regla de oro: **SIEMPRE usa `--body-file`**

**NUNCA** pases el body directo con `-b` o `--body` en PowerShell. Los caracteres `"` `'` `$` `(` `)` `{` `}` `&` `|` `>` `<` `;` rompen el parsing.

**SIEMPRE:**
1. Escribe el contenido a un archivo temporal (`.md`)
2. Usa `gh issue create --body-file archivo.md`

---

## Plantilla de issue (copia/pega en archivo `.md`)

```markdown
**Archivo:** `ruta/al/archivo.ext:líneas`

**Código problemático:**
```rust
// código real aquí
```

**Problema:** Explicación técnica clara (1-3 líneas).

**Solución:** Qué cambiar exactamente.

**Ejemplo real:** Situación concreta en tienda (usuario, impacto, consecuencia).
```

---

## Comandos exactos que funcionaron

```powershell
# 1. Crear archivo temporal con el body
$body = @"
**Archivo:** \`yarvis-app/src/App.tsx:140-160\`

**Código problemático:**
\`\`\`typescript
const checkSetup = async () => {
  const limite = Date.now() + 30_000;
  for (;;) {
    try {
      const setupDone = await verificarSetup();
      return;
    } catch (error) {
      if (Date.now() > limite) { ... }
      await new Promise((r) => setTimeout(r, 300));
    }
  }
};
\`\`\`

**Problema:** Reintenta cada 300ms fijos por 30s = 100 requests. Sin backoff exponencial ni feedback.

**Solución:** Backoff exponencial (300ms → 600ms → 1.2s → 2.4s → max 5s) + mensajes de progreso.

**Ejemplo real:** Laptop Celeron 4GB RAM. Migración 12s. Usuario ve spinner 30s → cree que se colgó → fuerza cierre → corrompe BD.
"@
$body | Out-File -Encoding utf8 issue_temp.md

# 2. Crear issue
$env:Path += ";C:\Program Files\GitHub CLI"
cd C:\Users\aleja\Downloads\Y.A.R.V.I.S.-POS
gh issue create --title "Título claro y específico" --body-file issue_temp.md

# 3. Limpiar
Remove-Item issue_temp.md
```

---

## Errores comunes y soluciones

| Error | Causa | Solución |
|-------|-------|----------|
| `unknown arguments ["..."]` | Comillas/paréntesis en body inline | Usar `--body-file` |
| `gh not recognized` | PATH no actualizado | `$env:Path += ";C:\Program Files\GitHub CLI"` |
| `not authenticated` | Falta `gh auth login` | Ejecutar `gh auth login` una vez |
| `repo not found` | Directorio wrong | `cd` a la raíz del repo git |

---

## Checklist antes de crear issue

- [ ] Título: `[tipo]: descripción corta` (ej: `backend:`, `frontend:`, `test:`)
- [ ] Archivo + líneas exactas
- [ ] Bloque de código real (```rust / ```typescript)
- [ ] Problema técnico en 1-3 líneas
- [ ] Solución concreta
- [ ] Ejemplo real de tienda (impacto dinero/tiempo)
- [ ] Guardado en `.md` → `--body-file`

---

## Referencia rápida de comandos útiles

```powershell
# Ver issues abiertos
gh issue list --limit 20

# Ver issue específico
gh issue view 29

# Cerrar issue
gh issue close 29 --comment "Fixed in commit abc123"

# Listar repos
gh repo list

# Ver repo actual
gh repo view --json nameWithOwner
```