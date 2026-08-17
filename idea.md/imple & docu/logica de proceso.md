primero una interfaz de primer inicio tipo asi dando las gracias y la bienvenida al pos, del lado derecho de la pantalla del primer inicio pediremos datos principales:
- nombre del administrador
- Nombre de la Tienda
- Contraseña
- conformacion de contraseña
- {boton de agregar empleado}
- {boton de iniciar sesion}

del lado izquierdo daremos las gracias con un mensaje sigificativo

GRACIAS POR ELEGIRNOS
  Y.A.R.V.I.S. POS!

[ ICONO DE TIENDA ]

Nos emociona acompañarte en
el crecimiento de tu negocio.

Nuestro sistema está diseñado
para mejorar tus ganancias y 
predecir tus ventas con exactitud.

Y.A.R.I.V.S. tu asistente local.



una vez hecho eso seguimos con la siguiente interfaz el boton de iniciar sesion no validara por lel momento solo pasara a la siguiente interfaz sin interezarle si hay datos o no en esta ocasion para la segunda interfaz solo editaremos el lado derecho colocando 2 botones:

- iniciar sesion como empleado 
- iniciar sesion como administrador

para el siguiente paso tenemos 2 interfaces que completar ahora tenemos que hacer las 2 al mismo tiempo recomiendo comenzar por la interfaz de administrador  aqui decidiremos como se vera nuestro pos es importante por que define la calidad de nuestro trabajo

configuramos nuestros botones principales para el dashboard admin

ventas <-  El sistema debe aprender de tus ventas y decirte cuanto vas a vender el proximo fin de semana, mes etc. con un margen de error, el sistema presente las predicciones con Intervalos de Confianza seran graficas complejas pero tambien

inventario <- aqui estaremos viendo todos los productos que tenemos registrados ademas de algunas sugerencias como Entradas, salidas, alertas de "stock bajo" y prediccion de comrpas (como sugerir que comprar en base a tus ventas y fechas festivas, temporadas.) y auditorías (inventario físico vs. sistema). 
y quedaria tipo asi 
__________________________________________________________________________________________________________________________
|                |                                                                                                        
|  Y.A.R.V.I.S.  |   INVENTARIO GENERAL                                                                               |
|  ADMIN PANEL   |_________________________________________________________________________________________________________________     |
|                                                                                                                     |
|  VENTAS        |   BUSCAR: [_____________]  |  FILTRO: [ Todos v ]  |  [+ AGREGAR]  [EDITAR] [ELIMINAR]             |
|                |                                                                                                    |
| >INVENTARIO    |  _________________________________________________________________________________________________ |
|                |  |                                                                                                 |  |
|  FINANZAS      |  |  PRODUCTO                     |  CANTIDAD  |  PRECIO COSTO  |  PRECIO VENTA  | MARGEN UTILIDAD  |  |
|                |  |-------------------------------|------------|----------------|----------------|------------------|  |
|  CLIENTES      |  |  Coca Cola 600ml              |     45     |    $ 15.00     |    $ 22.50     |    33.3 %        |  |
|                |  |  Papas Sabritas Orig. 45g     |     12     |    $ 12.50     |    $ 19.00     |    34.2 %        |  |
|  TICKETS       |  |  Agua Ciel 1L                 |     05     |    $ 08.00     |    $ 13.00     |    38.4 %        |  |
|                |  |  Galletas Oreo Paq. 6pzs      |     28     |    $ 10.00     |    $ 16.50     |    39.3 %        |  |
|  EMPLEADOS     |  |  Jugo Mango 500ml             |     30     |    $ 14.50     |    $ 21.00     |    30.9 %        |  |
|                |  |  Detergente Ariel 1kg         |     08     |    $ 26.00     |    $ 39.50     |    34.1 %        |  |
|  AJUSTES       |  |  Aceite Nutrioli 1L           |     15     |    $ 32.50     |    $ 46.00     |    29.3 %        |  |
|                |  |  Leche Alpura 1L              |     22     |    $ 19.50     |    $ 27.50     |    29.0 %        |  |
|________________|  |  Arroz Extra 1kg              |     40     |    $ 11.50     |    $ 18.00     |    36.1 %        |  |
|                |  |  Frijol Isadora 1kg           |     10     |    $ 24.00     |    $ 32.00     |    31.4 %        |  |
|  SALIR         |  |  Galletas Marias              |     55     |    $ 09.50     |    $ 14.00     |    36.6 %        |  |
|________________|  |_______________________________|____________|________________|________________|__________________|  |
toma en cuenta las acciones de los botones debes implementarlas de una ves las acciones de  FILTRO: [ Todos v ]  |  [+ AGREGAR]  [EDITAR] [ELIMINAR]   buscar no por que es mas compleja  y a la hora de darle clic a por ejemplo agregar no quiero que se abra una ventana emergente simplemente quiero que se pueda editar la tabla como si fuera un exel lo mismo para editar y eliminar en la parte de filtro  vas a poner el ordenen el que quieres ver los productos de mas barato a mas caro de mas caro a mas barato, de A-Z y de Z-A, de mas maregen de ganancia a menor margen de ganancia y de menor margen de ganancia a mayor margen de ganancia tratra de abreviarlo para que quepa bien en el cuatrito y tambien verifica el problema de que si trato de scrollear dentro de la grafica no se me trabe el scroll okey? mas abajo vas a poner la siguiente grafica: 

________________________________________________________________________________________________________
|              |                                                                                       |
| Y.A.R.V.I.S. | INVENTARIO GENERAL                                                                    |
|______________|_______________________________________________________________________________________|
|              |                                                                                       |
| >INVENTARIO     ________________________________________   ________________________________________  |
|              | |                                        | |                                        | |
| VENTAS       | |        ALERTA DE PRODUCTO BAJO         | |         PREDICCIONES DE COMPRA         | |
|              | |----------------------------------------| |----------------------------------------| |
| FINANZAS     | |                                        | |                                        | |   
|              | |  - Agua Ciel 1L ......... Quedan: 05   | |  PRODUCTO            | SUGERENCIA      | |
| CLIENTES     | |  - Detergente Ariel ..... Quedan: 08   | |  |-------------------|--------------|  | |
|              | |  - Frijol Isadora ....... Quedan: 10   | |  Coca Cola 600ml     | + 3 Cajas       | |
| TICKETS      | |  - Aceite Nutrioli ...... Quedan: 15   | |  Sabritas Original   | + 20 pzs        | | 
|              | |  - Leche Alpura 1L ...... Quedan: 22   | |  Cerveza 355ml       | + 5 Cajas       | |   
| EMPLEADOS    | |  - Arroz Extra 1kg ...... Quedan: 40   | |  Hielo 5kg           | + 10 pzs        | |
|              | |  - Jabon Zote ........... Quedan: 12   | |  Galletas Oreo       | + 15 pzs        | |
| AJUSTES      | |                                        | |                                        | |
|              | |                                        | |  * Basado en: PROFETA DE META + QWEN   | |
|______________| |________________________________________| |________________________________________| |
|              |                                                                                       |
| SALIR        |_______________________________________________________________________________________|


tomar en cuenta que estas graficas se iran actualizando OJO: esto no es para que pongas el contenido de una ves es para que te des una idea de como se debe de ver okey? por que luego pones ese contenido y luego lo tengo qaue borrar por que se ve gatisimo asi que asi es como se deberia de ver 

y como ultimo scroll se debe mostrar esta interfaz 
_________________________________________________________________________________________________________________
|              |                                                                                                  |
| Y.A.R.V.I.S. | INVENTARIO GENERAL                                                                               |
|______________|__________________________________________________________________________________________________|
|              |  _______________________________________________________________________________________________  |
| >INVENTARIO  | |                                                                                               | |
|              | |   CONCILIACIÓN: INVENTARIO FÍSICO VS SISTEMA                                                  | |
| VENTAS       | | |-------------------------------------------------------------------------------------------| | |
|              | | | PRODUCTO                |  INV. FÍSICO  |  SISTEMA  |  DIFERENCIA  |        ESTADO        | | |
| FINANZAS     | | |-------------------------|---------------|-----------|--------------|----------------------| | |
|              | | | Coca Cola 600ml         |      45       |     48    |     -3       |          🔴          | | |
| CLIENTES     | | | Sabritas Orig. 45g      |      12       |     12    |      0       |          🟢          | | |
|              | | | Agua Ciel 1L            |      05       |     04    |     +1       |          🔵          | | |
| TICKETS      | | | Oreo Paquete 6pzs       |      28       |     30    |     -2       |          🔴          | | |
|              | | | Ariel Power 1kg         |      08       |     08    |      0       |          🟢          | | |
| EMPLEADOS    | | |_________________________|_______________|___________|______________|______________________| | |
|              | |                                                                                               | |
| AJUSTES      | |   [ EDITAR ]  [ ACTUALIZAR INV. ]  [ ACTUALIZAR SISTEMA ]                                     | |
|______________| |_______________________________________________________________________________________________| |
| SALIR        |__________________________________________________________________________________________________|


recuerda que los datos no los debes de poner por que se ve naco y luego lo tengo que quitar es para que te des una idea de como debe funcinar el sistema 

finainzas <- aqui si veremos las graficas complejas y completas de cada seccion controlaremos todos los Cortes de caja (X y Z) visualizando todos los que se han hecho tambien las graficas de Utilidad bruta, Utilidad Operativa, Utilidad Neta, Utilidad Marginal. 

clientes <- aqui mostraremos la gestion de los clientes y pedidos como Quién me compra, cuánto me compra, cuanto me genera  y cuándo fue su última visita, ademas de ver los pedidos encargados por clientes generados por el empleado, visualizando el formato de pago, los productos, si se aplico algun descuento, los tipos de ganancia etc. con la opcion de crear facturas reportes   

tickets y facturacion <- todo lo que ha pasado por la impresora y todo lo que se ha facturado. aqui podremos visualizar todos los tickets que se han impreso a lo largo del tiempo  el promedio de venta por ticket el promedio de tickets por dia, cosas asi ademas de la visualizacion de los cortes de caja de igual manera mostrar todos los cortes de caja generados el promedio de estos al igual que graficas tanto para tickets como para cortes de caja y la importantisima facturacion electronica.

asi se veria la primera interfaz si sobra espacio solo si sobra espacio crea unas graficas de pastel para promedio venta tickets dia y promedio corte y un mini menu donde pueda establecer una meta de promedio venta una grafica por cada promedio y la encierras en un recuadro bonito. 

__________________________________________________________________________________________
| [Y] Y.A.R.V.I.S.     |  TICKETS Y FACTURACIÓN                                           |
|     ADMIN PANEL      |__________________________________________________________________|
|                      |                                                                  |
|  VENTAS              |   PROMEDIO VENTA        TICKETS / DÍA        PROMEDIO CORTE      |
|  INVENTARIO          |  [ $ 285.50 MXN ]      [  42 TICKETS  ]      [ $ 9,120.00 ]      |
|  FINANZAS            | ________________________________________________________________ |
|  CLIENTES            |                                                                  |
| > TICKETS <          |  .---------------------------.  .---------------------------.    |
|  EMPLEADOS           |  |   HISTORIAL DE TICKETS    |  |    HISTORIAL DE CORTES    |    |
|  Y.A.R.V.I.S.        |  |---------------------------|  |---------------------------|    |
|  AJUSTES             |  | Ticket001 .... $450.00 [>]|  | Corte #120 ... $8,500 [>]    |
|                      |  | Ticket002 .... $120.00 [>]|  | Corte #119 ... $9,200 [>]    |
|                      |  | Ticket003 .... $890.00 [>]|  | Corte #118 ... $7,800 [>]    |
|                      |  | Ticket004 .... $340.00 [>]|  | Corte #117 ... $9,450 [>]    |
|                      |  | Ticket005 .... $115.00 [>]|  | Corte #116 ... $8,900 [>]    |
|                      |  | Ticket006 .... $670.00 [>]|  | Corte #115 ... $9,000 [>]    |
|                      |  | Ticket007 .... $210.00 [>]|  | Corte #114 ... $8,700 [>]    |
|                      |  |                           |  |                           |    |
|                      |  | [   VER MÁS TICKETS v   ] |  | [   VER MÁS CORTES v   ] |     |
|  CERRAR SESIÓN       |  '---------------------------'  '---------------------------'    |
|______________________|__________________________________________________________________|

la segunda interfaz solo mostrara graficas 
__________________________________________________________________________________________
| [Y] Y.A.R.V.I.S.     |  TICKETS Y FACTURACIÓN                                           |
|     ADMIN PANEL      |__________________________________________________________________|
|                      |                                                                  |
|  VENTAS              |                                                                  |
|  INVENTARIO          |   RENDIMIENTO DE TICKETS  [[ 7 DÍAS v ]]                         |
|  FINANZAS            |  ______________________________________________________________  |
|  CLIENTES            |   # |              _             _                               |
| > TICKETS <          |     |             / \           / \      _                       |
|  EMPLEADOS           |     |      _     /   \   _     /   \    / \    _                 |
|  Y.A.R.V.I.S.        |     |  _  / \   /     \_/ \   /     \__/   \  / \                |
|  AJUSTES             |     |_/_\/___\_/___________\_/______________\/___\__________     |
|                      |       LUN   MAR   MIE   JUE   VIE   SAB   DOM                  |
|                      |                                                                  |
|                      |                                                                  |
|                      |                                                                  |
|                      |   FLUJO DE CORTES DE CAJA  [[ 7 DÍAS v ]]                        |
|                      |  ______________________________________________________________  |
|                      |   $ |                    _             _                         |
|                      |     |      _            / \           / \                        |
|                      |     |     / \    _     /   \    _    /   \                       |
|                      |     |  __/   \__/ \   /     \__/ \  /     \_________             |
|                      |     |_/____________\_/____________\/________________\_______     |
|                      |       LUN   MAR   MIE   JUE   VIE   SAB   DOM                  |
|                      |                                                                  |
|  CERRAR SESIÓN       |                                                                  |
|______________________|__________________________________________________________________|

esas graficas seran generadas con datos historicos de los tickets para un buen analizis 

__________________________________________________________________________________________
| [Y] Y.A.R.V.I.S.     |  TICKETS Y FACTURACIÓN                                           |
|     ADMIN PANEL      |__________________________________________________________________|
|                      |                                                                  |
|  VENTAS              |   PREDICCIÓN DE FLUJO DE CAJA  [[ PRÓXIMOS 7 DÍAS v ]]           |
|  INVENTARIO          |  ______________________________________________________________  |
|  FINANZAS            |   $ |                      . - ~ ~ ~ - .   <-- (Predicción)      |
|  CLIENTES            |     |                  . '               ' .                     |
| > TICKETS <          |     |          . - ~ '                       ' .                 |
|  EMPLEADOS           |     |  . - ~ '                                   ' ~ - .         |
|  Y.A.R.V.I.S.        |     |_/__________________________________________________\______ |
|  AJUSTES             |       HOY    +1D    +2D    +3D    +4D    +5D    +6D    +7D       |
|                      |                                                                  |
|                      |                                                                  |
|                      |   FORMAS DE PAGO                   ACCIONES RÁPIDAS              |
|                      |  __________________________       ____________________________   |
|                      |  |                        |       |                          |  |
|                      |  |      [ GRAFICA ]       |       |  [ GENERAR FACTURA ]     |  |
|                      |  |      [   DE    ]       |       |                          |  |
|                      |  |      [ PASTEL  ]       |       |  [ REPORTE GRAL. PDF ]   |  |
|  CERRAR SESIÓN       |  |________________________|       |__________________________|  |
|______________________|__________________________________________________________________|

la grafica de prediccion sera con profeta y mostrara 2 lineas la de prediccion y la del aproximado con un minimo y un maximo 

empleados <- desde aqui podremos gestionar quienes son los empleados nombre de los empleados, cuanto venden, ventas canceladas, con descuentos ventas, metas que les puedes poner para generar bonos, definir turnos, horarios, definir su salario semanal, para que puedas visualizar cuanto te estan generando y cuanto te esta costando mantenerlo, visualizar cuando hicieron cortes de caja x y z recordemos que el corte de caja Z es el cierre de turno, visualizar cuando iniciaron turno

### recuerda que las graficas historicas se muestran de la fecha actual hacia atras lo que el usuario eliga 7 dias atras 15 dias atras 1 mes atras 6 meses atras 1 año atras o perzonalizado y si son predicciones es de la fecha acual a lo que el usuario quiera 7 dias al futuro 15 dias al futoroo 6 meses 1 año o personalizado 

Y.A.R.V.I.S. <- AQUI SOLO PONDRAS LA INTERFAZ DEL CHAT BOT NO HARAS QUE FUNCIONE POR QUE FALTA CONECTAR EL MOTOR. 


ajustes <- aqui estaremos mostrando datos del sistema como el tema de colores de la interfaz podras elegir entre el basico blanco negro y gris o una paleta de colores que aun no he definido o el modo oscuro que aun no he definido la paleta de colres tampoco xd, ademas aca se colocaran los datos de inicio de sesion y datos de la tienda como la ubicacion, codigo postal, nombre de la tienda, nombre del dueño o administrador 

tipo asi la interfaz: 
__________________________________________________________________________________________________
  Y.A.R.V.I.S.  |________________________________________________________________________________|
  ADMIN PANEL   |                                                                                |
________________|   ______________________________    _________________________________________  |
                |  |                              |  |                                         | |
   VENTAS       |  |    IDENTIDAD DE LA TIENDA    |  |             TEMAS DE INTERFAZ           | |
   INVENTARIO   |  |______________________________|  |_________________________________________| |
   FINANZAS     |  |                              |  |                                         | |
   CLIENTES     |  |  NOMBRE DEL DUEÑO:           |  |  SELECCIONA UN ESTILO:                  | |
   TICKETS      |  |  [________________________]  |  |                                         | |
   EMPLEADOS    |  |                              |  |   ( ) CLARO                             | |
   Y.A.R.V.I.S. |  |  NOMBRE DE LA TIENDA:        |  |                                         | |
 [> AJUSTES <]  |  |  [________________________]  |  |   ( ) OSCURO                            | |
                |  |                              |  |                                         | |
                |  |  UBICACIÓN:                  |  |   ( ) COLOR                             | |
                |  |  [________________________]  |  |                                         | |
                |  |                              |  |_________________________________________| |
                |  |  CÓDIGO POSTAL (C.P.):       |                                              |
________________|  |  [____________]              |       CONTRASEÑA DEL ADMINISTRADOR:          |
                |  |______________________________|       [****************************]         |
  CERRAR SESIÓN |                                                                                |
________________|________________________________________________________________________________|

despues vas a hacer la parte de abajo agregando un scroll: la parte de abajo se deberia ver asi 
aqui esta seleccionado el modo  [ TICKET ] <- es un boton
__________________________________________________________________________________________________
  Y.A.R.V.I.S.  |_________________________________________________________________________________|
  ADMIN PANEL   |                                                                                 |
________________|   PARSEADOR DE TICKETS                                                          |
                |   ______________________________________________________________________________|
   VENTAS       |                                                                                 |
   INVENTARIO   |   NOMBRE TICKET:                                      SUBIR TXT:                |
   FINANZAS     |   [______________________________]                         📂                   |
   CLIENTES     |                                                                                 |
   TICKETS      |                                                                                 |
   EMPLEADOS    |   [ CATÁLOGO ]            [ TICKET ]             [ INSERTAR ]                   |
   Y.A.R.V.I.S. |                                                                                 |
 [> AJUSTES <]  |   PREVISUALIZACIÓN DEL TICKET:                                                  |
                |   ______________________________________________________________________________|
                |  |                                                                              |
                |  |  PRODUCTO            CANT.      PRECIO        TOTAL                          |
________________|  |  --------------------------------------------------                          |
                |  |  EJEMPLO ITEM 1       1         $10.00        $10.00                         |
  CERRAR SESIÓN |  |  EJEMPLO ITEM 2       2         $05.00        $10.00                         |
________________|  |______________________________________________________________________________|

aqui esta el de insertar que se deberia ver masomenos asi: 

__________________________________________________________________________________________________
  Y.A.R.V.I.S.  |_________________________________________________________________________________
  ADMIN PANEL   |
________________|   PARSEADOR DE TICKETS
                |   ______________________________________________________________________________
   VENTAS       |
   INVENTARIO   |   NOMBRE CARPETA:                                     SUBIR CARPETA:
   FINANZAS     |   [______________________________]                         📂
   CLIENTES     |
   TICKETS      |
   EMPLEADOS    |   [ CATÁLOGO ]            [ TICKET ]           [> INSERTAR <]
   Y.A.R.V.I.S. |
 [> AJUSTES <]  |   PREVISUALIZACION DE LA CARPETA:
                |   ______________________________________________________________________________
                |  |                                                                              |
                |  |  ARCHIVO                     ESTADO            TAMAÑO                        |
________________|  |  ----------------------------------------------------------                  |
                |  |  ticket_001.txt              LISTO              12 KB                        |
  CERRAR SESIÓN |  |  ticket_002.txt              CARGANDO           15 KB                        |
________________|  |______________________________________________________________________________|

y aqui de como se deveria ver los camvios con catalogo 
__________________________________________________________________________________________________
  Y.A.R.V.I.S.  |_________________________________________________________________________________
  ADMIN PANEL   |
________________|   PARSEADOR DE TICKETS
                |   ______________________________________________________________________________
   VENTAS       |
   INVENTARIO   |   BUSCAR PRODUCTO:                                    ACTUALIZAR CATÁLOGO:
   FINANZAS     |   [______________________________]                         📂
   CLIENTES     |
   TICKETS      |
   EMPLEADOS    |   [> CATÁLOGO <]            [ TICKET ]             [ INSERTAR ]
   Y.A.R.V.I.S. |
 [> AJUSTES <]  |   PREVISUALIZACION DEL CATALOGO:
                |   ______________________________________________________________________________
                |  |                                                                              |
                |  |  NOMBRE DE PRODUCTO              PRECIO COSTO          PRECIO VENTA          |
________________|  |  ----------------------------------------------------------------            |
                |  |  LECHE ENTERA 1L                 $18.50                $22.50                |
  CERRAR SESIÓN |  |  HUEVO DOCENA                    $28.00                $35.00                |
________________|  |______________________________________________________________________________|


logica de el empleado hacemos funcionar el boton de + agregar empleado cuando le demos click nos debe arrojar el campo de relleno de nombre del empleado y la contraseña que se le creo 

__________________________________________________________________________________________
|                                           |                                            |
|    GRACIAS POR ELEGIRNOS                  |           INICIO DE SESIÓN                 |
|    Y.A.R.V.I.S. POS!                      |       -------------------------            |
|                                           |                                            |
|       _________________________           |        Seleccione su perfil de             |
|      /                         \          |           acceso al sistema                |
|     |     [ ICONO DE TIENDA ]   |         |                                            |
|      \_________________________/          |        ____________________________        |
|                                           |       |                            |       |
|    Nos emociona acompañarte en            |       |   INICIAR SESIÓN COMO      |       |
|    el crecimiento de tu negocio.          |       |        ADMINISTRADOR       |       |
|                                           |       |____________________________|       |
|    Nuestro sistema está diseñado          |                                            |
|    para mejorar tus ganancias y           |        ____________________________        |
|    predecir tus ventas con exactitud.     |       |                            |       |
|                                           |       |   INICIAR SESIÓN COMO      |       |
|    Y.A.R.V.I.S. tu asistente de IA local. |       |          EMPLEADO          |       |
|                                           |       |____________________________|       |
|                                           |                                            |
|-------------------------------------------|           [+ AGREGAR EMPLEADO]             |
|                                           |                                            |
|   NUEVO REGISTRO:                         |    Nombre del Empleado:                    |
|   Asegúrese de que las                    |    [____________________________________]  |
|   contraseñas coincidan antes             |                                            |
|   de guardar.                             |    Crear contraseña de acceso:             |
|                                           |    [************************************]  |
|                                           |                                            |
|                                           |    Confirmar contraseña de acceso:         |
|                                           |    [************************************]  |
|                                           |                                            |
|                                           |             [ GUARDAR USUARIO ]            |
|___________________________________________|____________________________________________|

una ves tengamos esta primera interfaz es hora de comenzar con el backend y la DB PARA GUARDAR LOS DATOS
COMO PRIMEERA INSTANCIA los nombres del administrador y la contraseña que el cree ahora si se deben guardar en la db pasando por rust que es quien gestiona la db y una vez hayas guardado las contraseñas y los nombres en la db haz pruebas de testo por que la interfaz de primer inicio solo se debe mostrar una vez 

una vez presionado el boton de guardar usuario se deberia ver algo asi 
__________________________________________________________________________________________
|                                           |                                            |
|    GRACIAS POR ELEGIRNOS                  |           Configuración de Acceso          |
|    Y.A.R.V.I.S. POS                       |           Introduce los datos iniciales    |
|                                           |                                            |
|       _______                             |    ADMINISTRADOR                           |
|      |       |                            |    [ Nombre completo                  ]    |
|      |  [I]  |                            |                                            |
|      |_______|                            |    TIENDA                                  |
|                                           |    [ Nombre del negocio               ]    |
|    Nos emociona acompañarte en            |                                            |
|    el crecimiento de tu negocio.          |    CONTRASEÑA            REPETIR           |
|                                           |    [ ********** ]      [ ********** ]      |
|    Sistema diseñado para mejorar          |                                            |
|    tus ganancias y predecir               |    ____________________________________    |
|    ventas con exactitud.                  |   |                                    |   |
|                                           |   |  EMPLEADO REGISTRADO:              |   |
|    Y.A.R.V.I.S. tu asistente de IA local. |   |  [ ICONO ]  Peter Parker           |   |
|                                           |   |____________________________________|   |
|                                           |                                            |
|                                           |            [ + AGREGAR EMPLEADO ]          |
|                                           |                                            |
|                                           |            [ INICIAR SESIÓN -> ]           |
|___________________________________________|____________________________________________|

y al iniciar sesion y elegir el perfil del empleado se deberia mostrar la interfaz de donde pide la contraseña
__________________________________________________________________________________________
|                                           |                                            |
|    GRACIAS POR ELEGIRNOS                  |           INICIO DE SESIÓN                 |
|    Y.A.R.V.I.S. POS                       |           ACCESO AL SISTEMA                |
|                                           |                                            |
|       _______                             |      ________________________________      |
|      |       |                            |     |  PERFIL DE EMPLEADO            |     |
|      |  [I]  |                            |     |                                |     |
|      |_______|                            |     |________________________________|     |
|                                           |     |                                |     |
|    Nos emociona acompañarte en            |     |   CONTRASEÑA                   |     |
|    el crecimiento de tu negocio.          |     |   [ ........ ]                 |     |
|                                           |     |                                |     |
|    Sistema diseñado para mejorar          |     |      ____________________      |     |
|    tus ganancias y predecir               |     |     |                    |     |     |
|    ventas con exactitud.                  |     |     |   ENTRAR AL POS -> |     |     |
|                                           |     |     |____________________|     |     |
|    Y.A.R.V.I.S. tu asistente de IA local. |     |________________________________|     |
|                                           |                                            |
|                                           |                                            |
|___________________________________________|____________________________________________|

se deberia ver algo asi como cuando pide la contraseña al administrador, ahora vamos con la interfaz del empleado

en el lado izquierdo vamos a poner una sidebar empleado que contenga 

nueva venta <- aqui es la caja de cobro dentro de esta interfaz se mostraran las acciones e informacion basica como cobrar una venta con scaner y que se guarde en la db, cobrar una venta con busqueta por texto asistida por IA, cobrar una venta por cliente es decir crear un cliente registrar el pedido del cliente y cobrar la venta (solo si es alguien que compra de manera constante los mismos productos o haciendo algunos cambios en los productos)  en el lado izquierdo estara la side bar en la parte de abajo esta el boton de cobrar,  y un cuadro de recomendacion de la IA  de que podrias ofrecer con lo que llevas registrado (antes de cobrar) en la ventana tambien se mostraran lienas de como debe ser primero va 
cantidad de procto  (cant.) producto, total, descuento, total con descuento que el boton de cobrar en lugar de que diga cobrar diga la cantidad final osea la cantidad con descuento para cobrer tambien agregaremos la barra de busqueda con IA 

inventario <- aqui mostraremos el inventario de la tienda que de igual forma tendra una barra de busqueda asistida por IA el inventrario debe mostrar cant, producto, precio,  y con la opcion de un mini menu que diga ordenar por: y tener la opcion de ordenarlo de la A-Z, mas baratos- mas caros, mas caros- mas baratos, mas vendidos-menos vendidos y por el momento solo eso

tickets y cortes <-aqui podremos ver tododos los tickets que has generado como empleado y todos los cortes de caja que haz generado como empleado tanto corte de caja X como corte de caja Z ademas aqui se mostrara tus trurnos de entrada y salida cada vez que inicias sesion como empleado estas iniciando tu turno y cada vez que haces un corte de caja Z estas cerrando tu turno como empleado, entonces asi se registra el horario en el que estuviste trabajando. podras ver corte promedio que haces tu como empleado, el ticket promedio que vendes. y tambien podras visualizar cuanto le estas costando a tu patron de manera sutil no detallada solo de manera sutil

<- cahtbot tendras acceso al chatbot tambien para poder hacer consultas rapidas, preguntar si vale la pena hacerle descuento a tal producto o si podrias ofrecer algun combo

ajustes <- como empleado tambien tienes derecho a elegir diferentes temas para el pos el blanco con cgris y negro, el modo oscuro y el modo a color. 

### barra de arriba o barra superior

aqui se mostraran atajos para cobrar como con el atajo de la tecla F5 que seria bueno para cobrar lo que ya tienes enlistado el atajo de F6 seria para abrir la caja registradora el atajo F7 seria para activar la barra de busqueda automaticamente y el atajo de la tecla F8 seria para cobrar el pedido de algun cliente  ademas de mostrar la lista de los atajos tambien mostrara una barra de turno de la siguiente manera turno: 8:00 am----------------------------------5:00 pm
de 8 a 5:00pm seria el turno que definio el administrador pero la barra mostrara el horario en el que llegaste y saliste o en el que hiciste el corte Z la barra comenzara a llenarse en el momento que tu hayas iniciado sesion como empleado en la parte superior tambien se mostrara la barra de busqueda 

deberia quedar algo asi 
_________________________________________________________________________________________________________________________
|              |                                                                                                        |
|              |   [F5] Cobrar   [F6] Caja   [F7] Buscar   [F8] Atajos                              [EMPLEADO: ADMIN_01]|
|              |   TURNO: 08:00 AM [||||||||||||||||||||||||||-------------------------] 05:00 PM                       |
|    [LOGO]    |________________________________________________________________________________________________________|
|              |                                                                                                        |
|              |   CANT.      PRODUCTO                         TOTAL          DESCUENTO      TOTAL C/DESC.              |
|  [VENTAS]    |   ---------------------------------------------------------------------------------------              |
|              |   2.0        Leche Entera 1L                  $ 44.00        $ 4.40         $ 39.60                    |
|  INVENTARIO  |   1.0        Pan Integral                     $ 35.00        $ 0.00         $ 35.00                    |
|              |                                                                                                        |
|  TICKETS     |                                                                                                        |
|              |                                                                                                        |
|  CLIENTES    |                                                                                                        |
|              |                                                                                                        |
|  FINANZAS    |                                                                                                        |
|              |                                                                                                        |
|  EMPLEADOS   |                                                                                                        |
|              |                                                                                                        |
|  Y.A.R.V.I.S.|                                                                                                        |
|              |                                                                                                        |
|  AJUSTES     |                                                                                                        |
|              |________________________________________________________________________________________________________|
|              |                                                                                |                       |
|              |  💡 RECOMENDACIÓN IA: Sugerir café molido (frecuente)                          | COBRAR TOTAL: $ 74.60 |
|______________|________________________________________________________________________________|_______________________|

en este punto el archivo que tenemos de app.txs ya habra superado las 600 lineas y sera hora de dividir el admin y al empleado en diferentes carpetas el front-admin y el front-empleado ahi vivran las interfaces.

