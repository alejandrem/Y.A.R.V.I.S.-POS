// ============================================================
// fixtures — Datasets reales de cortes X y Z (formato estándar).
// Se usan como verdad en los tests del parser: si el formato
// estándar parsea y verifica, el cimiento está sano.
// ============================================================

/// Corte Z 54 del 01/01/2025 (ventas por artículo).
pub const CORTE_Z_EJEMPLO: &str = r#"p0   *** CORTE Z EN MONEDA:MXN***
EMPRESA, S.A. DE C.V.



*** Corte Z 54
ESTACION01 01/01/2025 11:25:57 a. m.
**Ingresos**

EFE Pago de clientes  $2,023.80
TAR Pago de clientes    $439.00
-----------------------------------
  Total de Ingresos:  $2,462.80

**Egresos**

-----------------------------------
   Total de Egresos:       $.00
-----------------------------------
-----------------------------------
   Total en caja:     $2,462.80
-----------------------------------
-----------------------------------
*********VENTAS DEL CORTE**********
Ventas 16%        :       $.00
Impuesto 16%      :       $.00
Ventas 10%        :       $.00
Impuesto 10%      :       $.00
-----------------------------------
Ventas gravadas   :       $.00
Impuesto          :       $.00
Ventas no gravadas:  $2,462.80

-----------------------------------
Redondeos         :       $.00
Total de ventas   :  $2,462.80

-----------------------------------
Ventas credito    :       $.00
-----------------------------------

**Cheques del dia**

-----------------------------------
Total de vales emitidos :      $.00

-----------------------------------

Total de vales de cambio :       $.00

-----------------------------------

-----------------------------------
 ******* Cobranza del dia *******
 PAGO
-----------------------------------
Ingresos por cobranza:       $.00


**Ventas por artículo**

JACK DNL HONEY 70 -       1 -   $535.00
BLACK AND WHITE O -       1 -   $321.00
MADRILE. GRANADIN -       1 -    $80.00
CENTENARIO REPOSA -       1 -   $330.00
MARLBORO ROJO 100 -       1 -    $80.00
NEGRA MODELO 1L C -       3 -   $136.80
VOLT BLUE LATA 37 -       2 -    $38.00
LATON NEGRA MOD 4 -       6 -   $180.00
BACAR. MANGO 750  -       1 -   $240.00
SANGRE TORO ORIGI -       1 -   $300.00
HAVANA CLU. 700 M -       1 -   $215.00
 CIGARROS SUELTOS -       1 -     $7.00
**Total ventas del dia   :   2,462.80
**Total venta en unidades:      20.00
-----------------------------
Clientes atendidos: 9"#;

/// Corte X 1 del 31/03/2026 (ventas por ticket + por cliente).
pub const CORTE_X_EJEMPLO: &str = r#"p0   *** CORTE X EN MONEDA:MXN***
EMPRESA, S.A. DE C.V.

Cajero: GENERAL


Corte X 1
ESTACION01 31/03/2026 09:35:08 p. m.
**Ingresos**

EFE Pago de clientes    $397.00
 04 TARJETA BANCARIA    $305.00
-----------------------------------
  Total de Ingresos:    $702.00

**Egresos**

-----------------------------------
   Total de Egresos:       $.00
-----------------------------------
-----------------------------------
   Total en caja:       $702.00
-----------------------------------
-----------------------------------
*********VENTAS DEL CORTE**********
Ventas 16%        :       $.00
Impuesto 16%      :       $.00
Ventas 10%        :       $.00
Impuesto 10%      :       $.00
-----------------------------------
Ventas gravadas   :       $.00
Impuesto          :       $.00
Ventas no gravadas:    $702.00

-----------------------------------
Redondeos         :       $.00
Total de ventas   :    $702.00

-----------------------------------
Ventas credito    :       $.00
-----------------------------------

**Cheques del dia**

-----------------------------------
Total de vales emitidos :       $.00

-----------------------------------

Total de vales de cambio :       $.00

-----------------------------------

-----------------------------------
 ******* Cobranza del dia *******
PAGO
-----------------------------------
Ingresos por cobranza:       $.00


**Ventas por ticket**

REM - 10834		                  94.00
REM - 10835		                 145.00
REM - 10836		                  20.00
REM - 10837		                  20.00
REM - 10838		                   9.00
REM - 10839		                 305.00
REM - 10840		                 100.00
REM - 10841		                   9.00
**Total ventas del dia :     702.00
-----------------------------



**Ventas por cliente**

 Cliente de mostrador     19
                 gravada   :      $0.00
                 No gravada:    $702.00
                 Total     :    $702.00
**Total ventas del dia   :    $702.00
**Total venta en unidades:      19.00
-----------------------------

Clientes atendidos: 8"#;
