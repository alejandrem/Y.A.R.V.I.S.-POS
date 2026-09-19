# Cuenta OpenCode vinculada — Y.A.R.V.I.S. POS

> El free tier de Zen solo responde a sesiones de usuario creadas en sus
> apps ("can only be used from within OpenCode", verificado con 16+
> sondas en vivo 2026-09-19): una `sk-` suelta siempre recibe 403 aunque
> sea válida. Este módulo vincula la CUENTA del dueño vía OAuth para que
> el chat hable como él, igual que el CLI oficial.

## 1. Flujo (una vez por instalación)

1. Admin abre YARVIS → Configurar modelos → OpenCode → **Conectar cuenta OpenCode**.
2. Se abre su navegador en `auth.opencode.ai` (PKCE S256 + loopback local,
   mismo patrón que `google.rs`): él se loguea ahí, YARVIS nunca ve su
   contraseña.
3. El code vuelve al loopback, el backend lo intercambia por access +
   refresh tokens y los guarda en `app_data_dir/zen_sesion.json` (0600).
4. Desde ahí, cada llamada a Zen con proveedor `opencode` lleva el token
   de la sesión (refrescado solo). Sin vincular, todo sigue como antes
   (la `sk-` del frente).

## 2. Comandos (`lib.rs`, +3 = 147 totales)

| Comando | Rol | Qué hace |
|---|---|---|
| `zen_estado` | operario | `{vinculado, email}` sin secretos (para la UI). |
| `zen_login` | admin | PKCE + loopback + navegador + intercambio + guardado. Devuelve el email. |
| `zen_salir` | admin | Borra la sesión del disco. |

## 3. Seguridad (misma que `api_keys.json`, issue #4)

- Solo ADMIN vincula/desvincula: los tokens gastan su cuota.
- El frontend NUNCA ve tokens: `chat.rs` los sustituye en backend
  (`clave_para_proveedor`) antes de llamar al proveedor.
- Refresh automático con 5 min de margen; sin refresh o expirado, se
  cae a la `sk-` como siempre (nada se rompe al desvincular).

## 4. Límites honestos

- Si Zen cambia su OAuth o cierra el loopback a terceros, `zen_login`
  falla con mensaje claro y el chat sigue con `sk-` + local.
- El free tier lo controla Zen en servidor: vinculado funciona como el
  CLI; si ellos lo restringen más, ni la sesión pasa.
- Tokens: si el dueño cambia su contraseña de OpenCode o revoca el
  acceso, hay que revincular (el refresh fallará y se usa la `sk-`).
