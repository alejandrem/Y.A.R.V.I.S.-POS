# Cómo funcionan los horarios (explicado facilito)

Este archivo explica cómo lleva el POS los horarios de los empleados, las horas extra y las barritas de colores. Sin palabras técnicas. Si sabes leer un reloj, lo vas a entender.

---

## 1. Tu horario normal

El patrón te pone un horario, por ejemplo **entrada 9:00, salida 17:00**.

En tu tarjeta de "Mi Turno" hay una barrita que siempre muestra ese horario en los extremos: **09:00 a la izquierda, 17:00 a la derecha**. Eso nunca cambia, es lo que te puso el patrón.

Lo negro que se va pintando es lo que llevas trabajado. La bolita blanca eres tú: marca a qué hora llegaste.

---

## 2. Si llegas temprano

* **Llegas hasta 15 minutos antes (ej: 8:45):** no pasa nada malo ni bueno en tus horas. Solo te sale un mensajito de "¡Felicidades, llegaste temprano!".
* **Llegas más de 15 minutos antes (ej: 8:44):** esos 15 minutos son de cortesía (no cuentan), pero lo demás SÍ cuenta como hora extra. Si llegaste 8:44, tienes 1 minuto extra. Si llegaste a las 6:00 AM a hacer hours extra antes de tu turno, todo ese tiempo cuenta (menos los 15 de cortesía).
* En la barra se ve así: tu bolita se recorre hacia atrás (a tu hora real) y aparece un **palito verde** marcando tu hora oficial de entrada.

## 3. Si llegas tarde

No pasa nada raro: tu bolita aparece a la hora que llegaste (ej: 9:05) y lo negro empieza desde ahí. Lo que no trabajaste no se pinta. (Que llegues tarde es otro tema con tu patrón, el sistema solo lo dibuja.)

## 4. Si te quedas después de tu salida

Si tu salida es 17:00 y sigues trabajando hasta las 18:00, esa hora cuenta como extra. En la barra aparece un tramo verde después de tu salida.

## 5. Si entras de noche (después de tu salida)

Ejemplo: tu turno era 9:00–17:00 pero entraste a las 21:41 a hacer horas nocturnas. Eso SÍ cuenta como extra: desde que entraste (21:41) hasta que termines. La barrita de extra se pinta toda verde creciendo poquito a poquito.

## 6. El corte Z cierra tu día

Cuando terminas, haces tu **corte Z**. Eso hace 3 cosas:

1. Suma todo lo que vendiste.
2. Reinicia tu contador a $0.00 para el día siguiente.
3. **Congela tus horas**: la barra y tus extras se quedan fijos a la hora del corte. Ya nada se mueve.

## 7. La segunda barrita (la verde)

Hay una segunda barra que SOLO aparece cuando tienes horas extra de verdad:

* Empieza en tu llegada y termina en tu salida oficial (o en tu corte Z de noche).
* El verde solo sale en el pedazo que sí es extra. Lo normal va en negro.
* Si no tienes extra, esta barra ni aparece. Así de simple.

## 8. "En curso" en el historial

En tu lista de días con extra, si el día de hoy todavía no tiene corte Z, en vez de inventar una hora de salida dice **"En curso"**. Tu salida se escribe sola cuando hagas tu corte Z.

## 9. Dos verdades importantes

* **Sin login no hay avance.** Si no has entrado hoy, la barra va en 0%. El reloj solo no es trabajo.
* **La hora la pone tu computadora.** El POS cree lo que diga el reloj de Windows. Si la computadora está mal de hora, todo sale mal. Por eso la computadora de la tienda debe tener la hora automática activada.

---

## 10. Cómo piensa Y.A.R.V.I.S. (tu ayudante de la tienda)

Imagina que Y.A.R.V.I.S. es un ayudante con muy buena memoria y varios cuadernos: uno de ventas, uno de inventario, uno de proveedores y otro de caducidades. Cuando le preguntas algo, primero piensa si necesita abrir un cuaderno. Si sí, lo abre, copia el dato exacto y con eso te responde. Nunca inventa números: si el cuaderno está vacío, te lo dice.

Tiene dos formas de pensar. Una es llamar por teléfono a un amigo muy listo que vive lejos: para eso la tienda necesita una llave y tener internet. La otra es pensar solito dentro de la computadora de la tienda, sin internet, aunque es más lento y más sencillo.

Si el amigo de lejos no contesta, no se queda callado: piensa solito y te avisa con un letrerito amarillo de que la respuesta vino de la tienda. Y siempre te dice quién respondió, para que sepas a quién creerle.
