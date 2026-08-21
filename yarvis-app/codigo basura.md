# Bitacora - Bug Metas Personalizadas (RESUELTO)

> ESTADO: El bug original ya esta resuelto. Este documento se conservan como
> bitacora historica. Las referencias a continuacion reflejan el estado ACTUAL
> del codigo (rutas, lineas y schema correctos), no los del momento del bug.

## Bug original (ya resuelto)
Las metas personalizadas no se guardaban / no aparecian en la UI despues de crearlas.

## Causa raiz
SQLite almacena valores numericos como INTEGER por defecto (ej: `0` en vez de `0.0`),
aunque la columna este definida como `REAL` en el schema. Cuando `sqlx::query_as` con
tipo `(f64, f64, ...)` intenta decodificar un INTEGER, falla con:

```
[Error] error occurred while decoding column 0: mismatched types;
Rust type `f64` (as SQL type `REAL`) is not compatible with SQL type `INTEGER`
```

## Archivos involucrados (rutas reales y actuales)

### Frontend
- `src/front-admin/ventanas/adminempleados/modalMetas.tsx`

### Backend (Rust)
- `src-tauri/src/backventanas/backadmin/adminempleados/modalmetas.rs`
  - `decode_f64()` (lineas 6-10): helper que resuelve el mismatch INTEGER→f64
  - `save_custom_goal` (lineas 180-200): INSERT de meta personalizada
  - `check_employee_goals` (lineas 218-332): SELECT + auto-completado de metas
  - `get_salario_info` (lineas 12-54): datos de salario del empleado
  - `get_employee_goals` (lineas 96-127): SELECT simple de metas
  - `save_employee_goal` (lineas 129-178): UPSERT de metas del sistema
- `src-tauri/src/backventanas/backadmin/adminempleados/empleados.rs` (dashboard)
- `src-tauri/src/backventanas/backadmin/adminempleados/modalempleado.rs` (CRUD empleado)
- `src-tauri/src/backventanas/backadmin/adminempleados/modalturnos.rs` (turnos)
- `src-tauri/src/models.rs` (structs `EmployeeGoal`, `SalarioInfo`)
- `src-tauri/src/lib.rs` (lineas 80-86: registro de comandos de metas)
- `src-tauri/migrations/0001_inicial.sql` (lineas 37-50: schema de `employee_goals`)

## Schema de la tabla `employee_goals`

> El schema NO vive en `db.rs`. Vive en migraciones SQL versionadas en
> `src-tauri/migrations/0001_inicial.sql` (lineas 37-50). `db.rs` solo ejecuta
> el migrador (ver `src-tauri/src/backventanas/db/db.rs` lineas 24 y 62-65).

```sql
-- src-tauri/migrations/0001_inicial.sql lineas 37-50
CREATE TABLE IF NOT EXISTS employee_goals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER NOT NULL,
    goal_type TEXT NOT NULL,
    goal_name TEXT,
    ventas_threshold TEXT DEFAULT '5',
    bonus_percentage REAL DEFAULT 0,
    bonus_amount REAL DEFAULT 0,
    is_completed INTEGER DEFAULT 0,
    completed_at TEXT,
    created_at TEXT DEFAULT (datetime('now','localtime')),
    FOREIGN KEY (employee_id) REFERENCES usuarios(id)
);
```

### Schema de la tabla `usuarios` (campos relevantes)

```sql
-- src-tauri/migrations/0001_inicial.sql lineas 24-34
    salario_semanal REAL DEFAULT 0,
    salario_diario REAL DEFAULT 0,
    dias_semana INTEGER DEFAULT 6
```

## Fixes aplicados (todos confirmados en el codigo actual)

### Fix 1: `decode_f64()` en el backend (RESUELTO)
`modalmetas.rs` lineas 6-10. Intenta decodificar como `f64`, y si falla intenta como `i64`:
```rust
fn decode_f64(row: &sqlx::sqlite::SqliteRow, col: &str) -> f64 {
    row.try_get::<f64, _>(col)
        .or_else(|_| row.try_get::<i64, _>(col).map(|v| v as f64))
        .unwrap_or(0.0)
}
```
Aplicado en: `get_salario_info` (linea 29), `get_employee_goals` (lineas 120-121),
`check_employee_goals` (lineas 260-261).
**Resultado**: El error de consola desaparecio.

### Fix 2: `handleAddCustom` recarga del backend (RESUELTO)
`modalMetas.tsx` lineas 171-185. Ahora invoca `save_custom_goal` y luego llama
`loadGoals()` (que invoca `check_employee_goals`) para recargar la lista completa
desde la DB en vez de inyectar un objeto local con ID falso:
```tsx
const handleAddCustom = async () => {
    if (!selectedId || !customName.trim() || customBonus <= 0) return;
    try {
        await invoke("save_custom_goal", {
            empleadoId: selectedId,
            goalName: customName.trim(),
            bonusAmount: customBonus,
        });
        await loadGoals();  // Recarga del backend
        setCustomName("");
        setCustomBonus(0);
    } catch (e) {
        console.error("Error guardando meta custom:", e);
    }
};
```

### Fix 3: Division de `empleados.rs` en 4 archivos (APLICADO)
Organizacion por responsabilidad:
- `empleados.rs` → dashboard (get_empleados, get_empleado_ventas, etc.)
- `modalempleado.rs` → CRUD (update_empleado, delete_empleado)
- `modalmetas.rs` → metas/salario (get_salario_info, save_salario, check_employee_goals, etc.)
- `modalturnos.rs` → turnos (get_turnos_empleados)

### Fix 4: Eliminar IIFE de `horasPorDia` y state `salarioInfo` (APLICADO)
`modalMetas.tsx` linea 65: `const [horasPorDia, setHorasPorDia] = useState(8);`
y en `loadSalarioInfo` (linea 87): `setHorasPorDia(info.horas_por_dia);`

### Fix 5: Validacion de `ventasBonusPct` (APLICADO)
Cambio de `if (v <= 10)` a `if (v >= 1 && v <= 10)` para evitar valores 0 o negativos.

### Fix 6: Clase CSS redundante (APLICADO)
Se elimino `${selectedId === emp.id ? 'text-neutral-400' : 'text-neutral-400'}`.

## Flujo actual del modulo (funcionando)

### Flujo de guardado de meta personalizada
1. Usuario escribe nombre y bono en los inputs
2. Usuario hace click en "+"
3. Frontend `handleAddCustom()` (`modalMetas.tsx` lineas 171-185):
   - Invoca `save_custom_goal` (`modalmetas.rs` lineas 180-200) que hace
     INSERT a `employee_goals` con goal_type='custom'
   - Invoca `loadGoals()` que llama `check_employee_goals` (`modalmetas.rs` lineas 218-332)
     que hace SELECT de todos los goals del empleado y devuelve la lista
4. El backend retorna la lista de goals (incluye los custom gracias al `_ => {}`
   en el match de lineas 308)
5. Frontend actualiza state `goals` con la respuesta (`modalMetas.tsx` linea 97)
6. `customGoals = goals.filter(g => g.goal_type === "custom")` (`modalMetas.tsx` linea 137)
   incluye la nueva meta

### Flujo de "Guardar Todo"
1. Frontend `handleSaveAll()` (`modalMetas.tsx` lineas 139-169):
   - Invoca `save_salario` (UPDATE `usuarios`)
   - Invoca `save_employee_goal` para "ventas" (UPSERT `employee_goals`)
   - Invokes `save_employee_goal` para "puntualidad" (UPSERT `employee_goals`)
   - Llama `onSaved()` (callback del padre)
   - Llama `onClose()` (cierra modal)

### Auto-completado de metas (bonus)
`check_employee_goals` (`modalmetas.rs` lineas 263-310) evalua si las metas del sistema
se cumplieron:
- "ventas": suma ventas de la semana del cajero y compara con `ventas_threshold`
- "puntualidad": compara `ultimo_login` contra `horario_inicio` (tolerancia +5 min)
Si se cumple, ejecuta UPDATE `is_completed = 1`. Las metas custom (`_ => {}`, linea 308)
no tienen logica de auto-completado: siempre permanecen pendientes hasta que se marquen
manualmente.

## Notas sobre las antiguas "pistas no exploradas"

Las siguientes hipotesis se listaron cuando el bug estaba activo. Su estado actual:

1. **"La DB real puede tener datos corruptos o viejos"** — DESCARTADA.
   El schema vive en migraciones versionadas (`0001_inicial.sql` lineas 37-50) que
   aplican con `IF NOT EXISTS`. La tabla `employee_goals` tiene todas las columnas
   (`bonus_percentage`, `bonus_amount`) desde la migracion inicial.

2. **"El INSERT de `save_custom_goal` podria estar fallando silenciosamente"** — DESCARTADA.
   El INSERT (`modalmetas.rs` lineas 189-197) usa `.map_err(|e| e.to_string())?` en el
   `.execute()`, asi que cualquier error de SQL se propaga como Err y el frontend lo
   captura en el catch (`modalMetas.tsx` linea 183).

3. **"`check_employee_goals` tiene logica de auto-completado que modifica la DB"** —
   CONFIRMADO PERO INOCUO. El auto-completado (`modalmetas.rs` lineas 263-310) solo
   ejecuta UPDATE sobre metas de tipo "ventas" y "puntualidad" si `!is_completed`.
   Las metas custom (`_ => {}`, linea 308) no se tocan. No hay riesgo de que el
   auto-completado afecte a las metas personalizadas.

4. **"El componente se desmonta y remonta"** — NO APLICA al guardado de metas custom.
   `handleAddCustom` no llama `onClose()`, asi que el modal permanece abierto y
   `selectedId` se conserva.

5. **"Posible race condition entre `loadGoals()` y `setGoals()`"** — TEORICO.
   `handleAddCustom` es secuencial (`await` en linea 179), asi que `loadGoals()` se
   ejecuta despues de que el INSERT se completa. No hay race condition en la practica.

## Conclusion

El bug original (mismatch INTEGER/f64 de SQLite) esta resuelto mediante `decode_f64()`
y el flujo de guardado/recarga del frontend. Las "pistas no exploradas" del documento
original han sido verificadas y descartadas o confirmadas como inocuas.
