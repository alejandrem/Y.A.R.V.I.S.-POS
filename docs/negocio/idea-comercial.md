# Idea a Vender del Software

> Estado 2026-08-26: venta rapida, inventario, clientes, reportes/cortes, tickets, parseador local, chat con tools y predicciones Holt-Winters ya implementados. Pendientes: busqueda semantica con modelo de embeddings propio, impresion termica y facturacion electronica, y fine-tuning final de Qwen2.5-Coder 1.5B Instruct. Ver implementacion.md.

## Venta Ultra-Rapida

El cajero no puede esperar. Debe soportar escaner de codigos de barras, busqueda rapida por teclado y multiples metodos de pago (efectivo, tarjeta, transferencia). La caja nunca depende de la IA: si el modelo no esta cargado, el cobro sigue funcionando.

## Control de Inventario Real

Entradas, salidas, alertas de stock bajo y prediccion de compras (sugerir que comprar en base a ventas y estacionalidad) y auditorias (inventario fisico vs sistema). Importacion masiva de catalogos con hash anti-duplicado y transaccion todo-o-nada.

## Gestion de Clientes (CRM Basico)

Quien compra, cuanto compra, cuanto genera y cuando fue su ultima visita. Pedidos por cliente con metodo de pago, productos, descuentos y tipo de ganancia.

## Reportes Financieros

Corte de caja X y Z, utilidad bruta, operativa, neta y marginal. Graficas por utilidad con impacto visual. Todo en centavos enteros para evitar errores de redondeo.

## Tickets y Facturacion

Impresion termica via drivers del sistema (ESC/POS, pendiente) y facturacion electronica legal (XML/PAC, pendiente). Historial de tickets y cortes con promedios y graficas de rendimiento.

## Prediccion de Ventas

El sistema aprende de las ventas y dice cuanto se vendera el proximo fin de semana o mes, con intervalos de confianza al 95%. Implementado localmente con Holt-Winters triple aditivo (src-ia/predicciones): estacionalidad semanal (m=7), ajuste por grid de 343 combos y banda que crece con raiz del horizonte. Sin Prophet ni servicios externos.

## Asistente Natural para el Dueno

Chatbot y deteccion de anomalias: muchos reembolsos de un mismo cajero o ventas a precios inusuales disparan alertas. El dueno pregunta "hubo algo raro hoy?" y la IA responde con datos reales via tools: "entre las 16 y 17h hubo 4 reembolsos del mismo producto por el cajero Juan, inusual para un martes".

Busqueda experta: "que productos para el cabello no tienen sal?" -> "tienes el shampoo marca X y el acondicionador Y". Un empleado nuevo da atencion experta desde el primer dia via search_products / list_categories / get_products_by_category.

Contador conversacional: "cuanto gane libre hoy quitando el costo?" -> "tu utilidad neta hoy fue de 2,543 MXN, 15% mas que el promedio de los miercoles". Predicciones de inventario: "que deberia comprar para el fin de semana?" -> "viene frente frio, tus ventas de cafe y pan suben 30% historicamente, aumenta 15-20% el pedido de pan".

El chat usa 10 tools de solo lectura, SQL parametrizado y roles (admin ve finanzas/nomina, empleado solo mostrador). Estrategia: fine-tuning de Qwen + ejecutor de tools, sin RAG.

## Parseador de Tickets de la Tienda

Cuando una tienda ya usa otro POS, sube hasta 12,000 tickets en TXT/CSV/Excel y YARVIS aprende su historia sin carga manual de productos o empleados. Importacion inteligente local: todo ocurre en el equipo por procesamiento masivo en lotes con streaming y transaccion por archivo, sin asfixiar la computadora.

## Sistema Portable y Auto-Sostenible

Un ejecutable, una base de datos, cero dependencias obligatorias. Peso estimado del bundle con modelo: ~5 GB. El sistema verifica RAM disponible antes de cargar el modelo local (RAM_GB_MINIMA_1_5_CODER = 1.0 GB en src-ia/motor-chat/llm/mod.rs:212); si no alcanza, responde por cloud o con error claro. No hay escalado automatico 0.5B/0.8B/1.5B Coder en esta version: hay un unico modelo local (Qwen2.5-Coder 1.5B Instruct fine-tuneado).

Modelo local con opcion de conectarse a internet. Si la nube falla (429 u otro), el chat hace fallback a local. El usuario siempre recibe respuesta.
