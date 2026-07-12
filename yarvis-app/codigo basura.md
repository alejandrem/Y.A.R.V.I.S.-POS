# Codigo Basura - Bug Metas Personalizadas

## Bug principal
Las metas personalizadas no se guardan / no aparecen en la UI despues de crearlas.

## Archivos involucrados

### Frontend
- `src/front-admin/ventanas/adminempleados/modalMetas.tsx`

### Backend (Rust)
- `src-tauri/src/backventanas/adminempleados/modalmetas.rs` (comandos: `save_custom_goal`, `check_employee_goals`, `get_salario_info`)
- `src-tauri/src/backventanas/adminempleados/empleados.rs` (dashboard)
- `src-tauri/src/backventanas/adminempleados/modalempleado.rs` (CRUD empleado)
- `src-tauri/src/backventanas/adminempleados/modalturnos.rs` (turnos)
- `src-tauri/src/models.rs` (structs `EmployeeGoal`, `SalarioInfo`)
- `src-tauri/src/lib.rs` (registro de comandos)
- `src-tauri/src/backventanas/db/db.rs` (schema de la DB, tabla `employee_goals`)

## Error original en consola
```
[Error] error occurred while decoding column 0: mismatched types;
Rust type `f64` (as SQL type `REAL`) is not compatible with SQL type `INTEGER`
```
Aparecia en `modalMetas.tsx:67` (linea de `loadSalarioInfo`).

## Causa del error SQL
SQLite almacena valores numericos como INTEGER por defecto (ej: `0` en vez de `0.0`),
aunque la columna este definida como `REAL` en el schema. Cuando `sqlx::query_as` con
tipo `(f64, f64, ...)` intenta decodificar un INTEGER, falla.

### Schema de la tabla `employee_goals` (db.rs linea 72)
```sql
CREATE TABLE IF NOT EXISTS employee_goals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER NOT NULL,
    goal_type TEXT NOT NULL,
    goal_name TEXT,
    ventas_threshold TEXT DEFAULT '5',
    bonus_percentage REAL DEFAULT 0,    -- <-- DEFAULT 0 es INTEGER, no REAL
    bonus_amount REAL DEFAULT 0,        -- <-- DEFAULT 0 es INTEGER, no REAL
    is_completed INTEGER DEFAULT 0,
    completed_at TEXT,
    created_at TEXT DEFAULT (datetime('now','localtime')),
    FOREIGN KEY (employee_id) REFERENCES usuarios(id)
)
```

### Schema de la tabla `usuarios` (db.rs lineas 61-67)
```sql
salario_diario REAL DEFAULT 0      -- DEFAULT 0 es INTEGER
dias_semana INTEGER DEFAULT 6
```

## Intentos de fix (todos aplicados, ninguno resolvio el bug)

### Fix 1: `decode_f64()` en el backend
Se creo una funcion helper que intenta decodificar como `f64`, y si falla intenta como `i64`:
```rust
fn decode_f64(row: &sqlx::sqlite::SqliteRow, col: &str) -> f64 {
    row.try_get::<f64, _>(col)
        .or_else(|_| row.try_get::<i64, _>(col).map(|v| v as f64))
        .unwrap_or(0.0)
}
```
Se aplico en: `get_salario_info`, `get_employee_goals`, `check_employee_goals`, `get_empleados`, `get_cortes_empleado`.
**Resultado**: El error de consola desaparece, pero las metas personalizadas siguen sin guardarse.

### Fix 2: `handleAddCustom` ahora llama `loadGoals()` despues de guardar
Originalmente creaba un objeto local con `id: Date.now()` (ID falso). Se cambio para
recargar del backend:
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
**Resultado**: No resolvio el bug.

### Fix 3: Division de `empleados.rs` en 4 archivos
Se dividio el archivo original para mejor organizacion:
- `empleados.rs` → dashboard (get_empleados, get_empleado_ventas, etc.)
- `modalempleado.rs` → CRUD (update_empleado, delete_empleado)
- `modalmetas.rs` → metas/salario (get_salario_info, save_salario, check_employee_goals, etc.)
- `modalturnos.rs` → turnos (get_turnos_empleados)

**Resultado**: Organizacion mejorada pero bug persiste.

### Fix 4: Eliminar IIFE de `horasPorDia` y state `salarioInfo`
Se reemplazo el IIFE que se re-ejecutaba en cada render con un state directo:
```tsx
const [horasPorDia, setHorasPorDia] = useState(8);
// En loadSalarioInfo:
setHorasPorDia(info.horas_por_dia);
```
**Resultado**: Mejora de performance pero no relacionado con el bug de metas.

### Fix 5: Validacion de `ventasBonusPct`
Se cambio `if (v <= 10)` a `if (v >= 1 && v <= 10)` para evitar valores 0 o negativos.
**Resultado**: Bug menor, no relacionado con metas personalizadas.

### Fix 6: Clase CSS redundante
Se elimino `${selectedId === emp.id ? 'text-neutral-400' : 'text-neutral-400'}`.
**Resultado**: Limpieza de codigo, no relacionado.

## Flujo actual del bug

### Flujo de guardado de meta personalizada
1. Usuario escribe nombre y bono en los inputs
2. Usuario hace click en "+"
3. Frontend ejecuta `handleAddCustom()`:
   - Invoca `save_custom_goal` en el backend (INSERT a `employee_goals`)
   - Invoca `loadGoals()` que llama `check_employee_goals` (SELECT de `employee_goals`)
4. El backend retorna la lista de goals
5. Frontend actualiza state `goals` con la respuesta
6. `customGoals = goals.filter(g => g.goal_type === "custom")` deberia incluir la nueva meta

### Flujo de "Guardar Todo"
1. Frontend ejecuta `handleSaveAll()`:
   - Invoca `save_salario` (UPDATE `usuarios`)
   - Invoca `save_employee_goal` para "ventas" (UPSERT `employee_goals`)
   - Invoca `save_employee_goal` para "puntualidad" (UPSERT `employee_goals`)
   - Llama `onSaved()` (callback del padre)
   - Llama `onClose()` (cierra modal)

## Pistas no exploradas

### 1. La DB real puede tener datos corruptos o viejos
Si la tabla `employee_goals` fue creada con un schema viejo que no tenia `bonus_percentage`
o `bonus_amount`, esas columnas podrian no existir. Los `ALTER TABLE` en `db.rs` no
agregan columnas a `employee_goals` (solo a `usuarios`).

### 2. El INSERT de `save_custom_goal` podria estar fallando silenciosamente
El INSERT usa 4 columnas pero la tabla tiene 10. Las columnas sin especificar usan defaults.
Si algun default falla o hay un constraint, el INSERT podria fallar sin throw.

### 3. `check_employee_goals` tiene logica de auto-completado que modifica la DB
Dentro del SELECT, si una meta de ventas o puntualidad se cumple, ejecuta un UPDATE.
Esto podria causar problemas si hay concurrent access o si el UPDATE falla.

### 4. El componente se desmonta y remonta
Cuando `handleSaveAll` llama `onSaved()` + `onClose()`, el componente se desmonta.
Si el padre re-renderiza y el componente se remonta, `selectedId` vuelve a `null`
y `loadGoals()` no se ejecuta.

### 5. Posible race condition entre `loadGoals()` y `setGoals()`
`handleAddCustom` llama `loadGoals()` que es async. Mientras se resuelve, el usuario
podria hacer otra accion que modifique `goals`.

## Que falta por intentar

1. **Verificar directamente en la DB** si el INSERT realmente crea la fila:
   ```sql
   SELECT * FROM employee_goals WHERE goal_type = 'custom';
   ```

2. **Agregar logging** en `save_custom_goal` para ver si el INSERT se ejecuta:
   ```rust
   println!("INSERTando meta custom: empleado={}, nombre={}, monto={}", empleado_id, goal_name, bonus_amount);
   ```

3. **Agregar logging** en `check_employee_goals` para ver que retorna:
   ```rust
   println!("Goals encontrados: {}", result.len());
   ```

4. **Verificar que el `invoke` del frontend** realmente llega al backend
   (agregar `console.log` antes y despues del invoke).

5. **Verificar que `loadGoals()`** se ejecuta despues del `invoke("save_custom_goal")`
   y que no hay error atrapado en el catch.
