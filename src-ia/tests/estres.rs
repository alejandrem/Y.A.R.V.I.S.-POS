//! estres.rs — Suite de ESTRÉS para destrozar el parseador de tickets.
//!
//! No son tests de "funciona": son ataques directos de fuzzing y dardos a las
//! invariantes. Objetivo: que CUALQUIER entrada (basura, gigante, maliciosa o
//! alucinada) NUNCA produzca panic, NaN/inf ni dinero absurdo en la BD.
//!
//! Invariantes que se vigilan:
//!   1. Ninguna función panfique con entrada arbitraria (incl. UTF-8 roto).
//!   2. Los precios/cantidades SIEMPRE son finitos y con magnitud <= 1e12.
//!   3. Método de pago SIEMPRE cae en el conjunto conocido.
//!   4. El dinero del ticket cuadra: total DB == subtotal + IVA (redondeo 2).
//!
//! Distribución:
//!   - `soporte`  → helpers compartidos (mapeo, finito, tmp_workspace, BD).
//!   - `fuzzing`  → los 9 tests del parser contra texto hostil.
//!   - `masivo`   → los 4 tests del parseador_masivo a escala.

mod fuzzing;
mod masivo;
mod soporte;
