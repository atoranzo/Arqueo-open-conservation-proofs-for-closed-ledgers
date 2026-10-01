#!/usr/bin/env bash
# =========================== EL CANON ===========================
#
# **La fuente unica.** Antes de §224 cada BLOQUE reescribia sus compuertas
# a mano, y por eso `zk-ssl-wire` paso SEIS sellos sin que nadie corriera
# sus tests.
#
#     bash tools/canon.sh --sello      # cada bloque · **151 s**
#     bash tools/canon.sh --largo      # + halo2 y plonk · **~20 min**
#     bash tools/canon.sh --completo   # + zk-core · **53,6 min**
#
# ⚠️ Estos tiempos los midio el CANON, no los bancos (§226, primera
# pasada de `--completo`): 151 + 347 (halo2) + 689 (plonk) + 2.032
# (zk-core) = **3.217 s**. Los bancos F.1 y G.2 daban cifras mayores
# porque midieron en FRIO; el canon mide con el `target` caliente,
# que es como se corre de verdad. La diferencia no es pequeña:
# zk-core 2.032 s aqui frente a los 2.317 que sumaban las dos
# invocaciones separadas de G.2.
#     bash tools/canon.sh --lista      # solo enseña la tabla, no ejecuta
#     bash tools/canon.sh --bancos     # los bancos, todos (tools/bancos.sh) · **~15 min**
#
# ## Lo que lo hace ROBUSTO, y no es la tabla
#
# 1. **Los miembros del workspace se LEEN de Cargo.toml.** Un crate nuevo
#    sin fila pone la compuerta ROJA, y una fila sin crate tambien. El
#    agujero de §223 pasa de "hay que acordarse" a **imposible**.
# 2. **Pines EXACTOS, no minimos.** Nada mejora ni empeora en silencio.
# 3. **No se para en el primer fallo:** da el inventario completo.
# 4. **Timeout POR CRATE.** Un crate que cuelgue no cuelga la compuerta.
# 5. **Anclas ASCII.** `.` de grep casa un BYTE: un ancla con tilde no
#    falla, se queda MUDA (§223).
# 6. **Here-string, nunca `echo | grep`**: con `pipefail`, `grep -q` sale
#    al encontrar, `echo` recibe SIGPIPE y devuelve 141 aunque la busqueda
#    acierte. Una compuerta intermitente es peor que ninguna.
# 7. **Dice que linea editar** cuando un pin no cuadra.
# 8. **Ningun `.rs` bajo `src/` puede quedar sin declarar** (§227). Lo que
#    no se compila no se verifica — y encima invita a especular.
#
# ## Los TRES niveles, y por que tres (§225)
#
# `zk-core` cuesta **33,9 minutos** medidos por el canon (2.032 s). G.2
# lo midio en dos invocaciones separadas y en frio: 1.472 s los 73 tests
# de biblioteca con 8 hilos mas **845 s la ceremonia**. Y no hay hilo
# que lo arregle: medido, generar la prueba añade un 5 % y rayon un
# 17 % — **el coste es SINTETIZAR el circuito**, y `ark-r1cs-std`
# sintetiza en serie.
#
# Meter 54 minutos en cada sello garantizaria que alguien acabe saltandose
# la compuerta, y una compuerta que se salta no protege. Es exactamente lo
# que le paso a `check_tests.py`.
#
# ⚠️ **El nivel `--completo` NO lo fuerza nadie.** Es disciplina, no
# compuerta, y la disciplina es justo lo que ha fallado seis veces en este
# proyecto. Lo unico que se puede hacer por construccion es **no fiarlo a
# la memoria**: cada `--completo` que pasa deja constancia en
# `.canon/ultimo-completo`, y TODA invocacion dice cuando fue y cuantos
# sellos han pasado desde entonces.
#
# ⚠️ **Los bancos, tampoco (§582).** Viven fuera de los niveles: arrancan
# el nodo, el testigo y el mando reales. `--bancos` los corre todos
# (`tools/bancos.sh`), cada pasada VERDE deja constancia en
# `.canon/ultimo-bancos`, y TODA invocacion dice cuando fue y si lo que
# ejercen -`crates/`, los `Cargo.*` y los propios bancos- cambio desde
# entonces. El de la reutilizacion estuvo ROJO del §337 al §579 sin que
# nada lo dijera.
#
# ## Como se actualiza un pin
#
# Se MIDE primero y se edita la tabla despues. Nunca al reves.
#
# ================================================================

# ⚠️ La guarda va ANTES de cualquier bashismo: con `sh` el `set -o pipefail`
# de abajo revienta primero y el aviso no llega a imprimirse nunca.
if [ -z "${BASH_VERSION:-}" ]; then
  echo "canon.sh necesita bash: usa 'bash tools/canon.sh', no 'sh'." >&2
  exit 2
fi

set -uo pipefail

# La raiz sale de la ruta del propio fichero. `CANON_RAIZ` la sobreescribe,
# y existe para PROBAR esta herramienta desde fuera del repo: una copia
# mutilada en /tmp resolveria su raiz a /tmp y no vigilaria nada. Una
# compuerta que no se puede probar en negativo no es una compuerta.
RAIZ="${CANON_RAIZ:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
cd "$RAIZ" || exit 2
NIVEL="${1:---sello}"
SELLO_FILE=".canon/ultimo-completo"
BANCOS_FILE=".canon/ultimo-bancos"

# ── LA TABLA ────────────────────────────────────────────────────
# crate | nivel | pasan | ignorados | warnings | timeout_s | nota
#
# Medido en §224 (banco F.1) y corregido en §225 (bancos G.1 y G.2) sobre
# el sello 06106c9, en release. `pasan` e `ignorados` son lo que el arnes
# EJECUTA, no los `#[test]` declarados.
TABLA=$(cat <<'FIN_TABLA'
zk-ssl             sello     431   7   0   600  alias=capa · la capa · §292: 262 -> 263, la cima en el digest · §304: 263 -> 264, el atado de la cifra publicada · §340: 264 -> 265, la altura entra en el digest · §345: 265 -> 269, el compositor v2 (RFC-0003, E1a) · §350: 269 -> 272, el cobro aprende el sobre (E3b-1 del RFC-0003) · §351: 272 -> 276, la apertura en el recibo (E3b-2 del RFC-0003) · §353: 276 -> 279, la via viva aprende el sobre (E3c-1b del RFC-0003) · §355: 279 -> 284, la guarda de ancho llega a los gemelos · §357: 284 -> 285, el testigo funcional de la des-emision (deuda del §351) · §359: 285 -> 286, el export del snapshot falla cerrado · §362: 286 -> 287, el reloj sale del atado a su contrato propio · §379: 287 -> 289, la conservacion del dinero se comprueba al abrir · §387: 289 -> 292, la raiz del pendiente se guarda y se comprueba al abrir · §388: 292 -> 294, la raiz de meta del pendiente se guarda y se comprueba al abrir · §390: 294 -> 295, descongelar sobrevive al reinicio · §391: 295 -> 298, la raiz de congelados en reposo · §392: 298 -> 301, la raiz del registro en reposo · §393: 301 -> 304, los contadores custodiados los deriva el registro · §394: 304 -> 307, la cuota de custodios la deriva el registro · §413: 307 -> 317, el consumo publicado (RFC-0006, E1) · §415: 317 -> 318, la pareja de consumos en el digest (RFC-0006, E2b) · §416: 318 -> 323, el camino del consumo (RFC-0006, E3a) · §452: 323 -> 326, la familia de v5 en la cabeza de la capa (RFC-0007, E1b) · §454: 326 -> 329, la causa de un rechazo como dato (RFC-0007, E2) · §458: 329 -> 330, el camino de congelados en la capa (RFC-0007, E3b) · §461: 330 -> 335, el instrumento de la puerta de la prueba de edad (RFC-0007, E4a) · §463: 335 -> 337, la prueba de edad real sobre el libro de la capa (RFC-0007, E4b-1) · §465: 337 -> 340, la prueba de edad se enlaza a la cabeza del libro (RFC-0007, E4b-2) · §466: 340 -> 344, el productor de la prueba de edad (RFC-0007, E4b-3) · §474: 344 -> 345, el camino de cuentas y su atado (RFC-0007, E5, 3b) · §478: 345 -> 350, el productor de la banda del saldo (RFC-0007, E5, corte 4b) · §485: 350 -> 354, el instrumento del cobro pendiente portable (RFC-0008, E1) · §487: 354 -> 357, la no-congelacion la impone el aplicador, no la prueba (5.A-264, A) · §491: 357 -> 365, el productor del cobrador (RFC-0008, E1) · §492: 365 -> 372, la foto de los pendientes que toma el latido (RFC-0008, D-F) · §495: 372 -> 373, el enlace del cobro pendiente en el productor de la capa (RFC-0008, E1) · §502: 373 -> 378, el instrumento del pago en curso portable (RFC-0008, E2) · §504: 378 -> 389, el productor del pago en curso en la capa (RFC-0008, E2) · §518: 389 -> 398, el productor de la prenda en la capa, con la clave del prendador (RFC-0008, E3) · §529: 398 -> 399, el testigo de la guarda de forma: la tupla entera, antes de construir el AIR, en los quince verify de la capa (5.A-381) · §531: 399 -> 421, el testigo de lo que revela una prueba: las 21 filas de D-B y el censo del cable (RFC-0009, E2) · §570: 421 -> 422, la pareja de recepcion entra en el digest y la cabeza compone v6 (RFC-0010, E2d) · §578: 422 -> 423, toda verificacion viva de la capa lee con sal: los quince y la pareja umbral · §641: 423 -> 424, la cota nativa de la banda en verify_audit, la atadura de identidad en circuit_audit y la aritmetica comprobada del envio/cobro/quema (RFC-0017) · §651: 424 -> 425, una prueba oculta con el meta vaciado es Err y no panico, en la guarda de forma y en apply_send (D3-META) · §653: 425 -> 426, una prueba con bytes de cola no verifica (SEG-04) · §654: 426 -> 428, una prueba del titular ya aplicada no vuelve a valer: tras un reembolso y tras un ciclo A->B->A (SEG-03) · §659: 428 -> 429, una prueba maleada por dentro -un nodo de mas en el lote de Merkle- no se reenvia tras un ciclo A->B->A (SEG-03b) · §668: 429 -> 431, el tope de suministro de la capa es 2^62 - 1: new y open lo exigen
stark-experiment   sello     407  13   0   600  alias=circuitos · los circuitos · §345: 297 -> 299, el gemelo nativo v2 (RFC-0003, E1a) · §346: 299 -> 304, RefundAirV2 en paralelo (E2a del RFC-0003) · §347: 304 -> 310, ClaimAirV2 en paralelo (E2b del RFC-0003) · §348: 310 -> 312, la puerta de E2 saldada (ladron-con-aviso y tercero-como-retorno) · §352: 312 -> 318, el envio aprende el sobre (E3c-1a del RFC-0003) · §463: 318 -> 328, el probador de la prueba de edad (RFC-0007, E4b-1) · §465: 328 -> 330, la cabeza fija la marca y el seq de la prueba de edad (RFC-0007, E4b-2) · §477: 330 -> 337, el probador de la banda del saldo (RFC-0007, E5, corte 4a) · §478: 337 -> 338, la boca del probador de la banda (RFC-0007, E5, corte 4b) · §490: 338 -> 348, el probador del cobro pendiente (RFC-0008, E1) · §495: 348 -> 350, la cabeza fija las raices y el nacido del cobro (RFC-0008, E1) · §503: 350 -> 364, el probador del pago en curso y sus catorce (RFC-0008, E2) · §511: 364 -> 379, el atado del camino de congelados a la cuenta (5.A-272, arreglo B) · §517: 379 -> 390, el probador de la prenda y sus once testigos, con los ignorados 12 -> 13 por su instrumento (RFC-0008, E3) · §529: 390 -> 391, el testigo del par ante una prueba de forma ajena, con la variante WrongTraceWidth (5.A-381) · §532: 391 -> 394, la foto del probador pristino: tres KAT con entradas fijas (RFC-0009, E3a-0) · §534: 394 -> 401, los siete falsadores de la ocultacion del nucleo, con el modo oculto solo en ellos (RFC-0009, E3a-2) · §537: 401 -> 403, los dos falsadores de D-AH: el probador oculto devuelve Err, no panico, con testigo malo o con m que no cabe (RFC-0009) · §641: 403 -> 404, la banda no se atribuye a un public_id ajeno, y el rango de los AIR baja a 62 bits, cerrando el wraparound (RFC-0017) · §651: 404 -> 405, el par umbral sin marca da WrongTraceWidth y no panico (D3-META) · §652: 405 -> 407, las semillas de la ocultacion son de 32 bytes: el ultimo byte mueve la prueba y la de produccion es fresca (CRIPTO-01)
ceremony           sello      34   0  11   300  alias=ceremonia · DEUDA: 11 warnings, pinchados para que no crezcan
settlement-layer   sello      17   0   0   300  alias=liquidación
iso-bridge         sello       3   0   0   300
zk-ssl-sdk         sello      15   0   0   300  §542: 6 -> 11, la prenda desde el Wallet, con la clave que no sale del crate (RFC-0008 E3, D-BH) · §550: 11 -> 13, el lector del keystore nombra el fichero y comprueba sus permisos (5.A-403, 5.A-404) · §606: 13 -> 15, el titular guarda la constancia -el recibo y el acuse de la respuesta, tal como llegaron- (la 109)
zk-ssl-wire        sello      26   0   0   180  §259: 20 metodos, y el json publicado atado a la tabla · §311: 3 -> 7, el DTO de la cabeza firmada · §312: 7 -> 10, el accesor falible de la cabeza firmada · §313: 10 -> 13, los tres constructores nombrados · §315: 13 -> 15, la cofirma en el cable · §415: 15 -> 17, la cabeza v4 en el cable y la vista por version (RFC-0006, E2b) · §434: 17 -> 19, del cable a la cabeza (RFC-0006, E4b-1b-i) · §452: 19 -> 22, la cabeza v5 en el cable y firmada() por version (RFC-0007, E1b) · §533: timeout 60 -> 180; su clausura lleva el probador por path y, tras un cambio en winter-*, el cable paga ~60 s de compilacion antes de sus 2 s de tests (exit 124 en el primer BLOQUE-533) · §570: 22 -> 23, la cabeza v6 en el cable y la vista por version (RFC-0010, E2d) · §641: 23 -> 24, el saldo del cliente se lee canonico en el cable (RFC-0017) · §650: 24 -> 25, el DATA del cable es el hex canonico, un multibyte es BadHex y no un panico · §662: 25 -> 26, la QUANTITY del cable sin +, sin mayusculas y sin ceros a la izquierda
zk-ssl-cli         sello      132   0   0   120  alias=testigo · §250: 22 -> 26, la vista dividida EN FRIO · §294: 28 -> 39, la historia como segundo canal · §295: 39 -> 43, el banco del consumidor · §299: 43 -> 51, el cofirmante del testigo · §300: 51 -> 54, el mando de la cofirma · §301: 54 -> 58, la herramienta del tercero · §310: 58 -> 59, la serie de cofirmas es por testigo · §312: 59 -> 61, el testigo consume la cabeza tipada · §316: 61 -> 62, el testigo envia su cofirma al nodo · §319: 62 -> 66, la politica que nombra y la k que falla cerrada · §320: 66 -> 71, la recoleccion de cofirmas · §314: 71 -> 75, la ausencia de respuesta es una clase · §333: 75 -> 79, la clave de la serie pasa al indice embebido · §334: 79 -> 81, la marca solo se dispara sobre lo que prueba · §336: 81 -> 86, la politica del cofirmante sale de fn run y nace con tests · §337: 86 -> 91, el testigo vuelve a arrancar tras cofirmar · §404: 91 -> 92, otra version no se cree · §406: 92 -> 93, el testigo lee la version por VersionCabeza antes de la firma · §409: 93 -> 94, el mando de --respuesta y el texto del veredicto · §411: 94 -> 95, los KAT del nucleo · §414: 95 -> 96, el canal de la historia ve la v4 (RFC-0006, E2a) · §415: 96 -> 97, el diario custodia la pareja de consumos (RFC-0006, E2b) · §451: 97 -> 98, el testigo recompone la v5 (RFC-0007, E1a) · §452: 98 -> 100, el diario custodia la v5 y su lista es la del cable (RFC-0007, E1b) · §497: 100 -> 107, la boca del cobrador, prueba-cobro (RFC-0008 E4-cobro, corte A) · §507: 107 -> 113, la boca del pagador, prueba-pago (RFC-0008 E2, D-AI) · §543: 113 -> 120, la boca del prendador, prueba-prenda, y la puerta de clausura del cli (RFC-0008 E3, D-BC..D-BI; 5.A-359) · §544: 120 -> 124, el keystore del receptor como quinto fichero de la siembra (RFC-0008 E3) · §559: 124 -> 125, la guarda del cobro pasa al predicado de la familia (RFC-0010 E2a) · §594: 125 -> 128, el gate de las cofirmas en todo estado que lea un contador (ECST §8.1): un positivo, dos rojos · §635: 128 -> 131, el juez del umbral del medio con la politica del cliente: cuenta testigos nombrados y distintos, no cuenta los que no verifican ni los repetidos, y falla cerrada bajo k y con k = 0 (RFC-0013 D-D) · §650: 131 -> 132, el testigo no cae ante un hex con un multibyte
zk-ssl-node        sello      188   0   0   400  alias=nodo · §261: 51 -> 56, la credencial para los caminos · §292: 79 -> 80, la cima en el digest · §293: 80 -> 82, la extension servida · §296: 82 -> 73, el guardian se muda a su crate · §298: 73 -> 71, la lectura del indice se muda · §302: 71 -> 72, el contrato publicado atado al despacho · §309: 72 -> 73, el conjunto de claves servidas, declarado · §315: 73 -> 76, el transporte de la cofirma · §317: 76 -> 77, la retencion de las cofirmas, atada · §318: 77 -> 80, el disparador de la acumulacion · §328: 80 -> 83, el arranque reconcilia · §331: 83 -> 84, la clave en cero no arranca · §335: 84 -> 91, la clave vuelve a su indice y el contador se ata al diario · §417: 91 -> 95, el consumo por el cable (RFC-0006, E3a) · §436: 95 -> 98, la puerta de libros ajenos (RFC-0006, E4b) · §440: 98 -> 100, los dos testigos de la frontera (RFC-0006, E4b-2) · §452: 100 -> 101, el nodo sirve la cabeza v5 y lo servido recompone (RFC-0007, E1b) · §454: 101 -> 104, la causa en el cable y en la negativa del consumo (RFC-0007, E2) · §458: 104 -> 108, el camino de congelados y dev_freeze (RFC-0007, E3b) · §493: 108 -> 112, la foto en el latido y zkssl_pendingPath (RFC-0008, D-F) · §505: 112 -> 115, la puerta del pagador en zkssl_pendingPath (RFC-0008 E2, D-AE) · §519: 115 -> 122, el cable de la prenda, zkssl_pledge (RFC-0008, E3) · §530: 122 -> 125, la red del panico: la PARADA con causa y sus tres testigos (5.A-382) · §559: 125 -> 126, la guarda del nodo sale a su fn y pregunta por el predicado (RFC-0010 E2a) · §565: 126 -> 137, el registro de recepcion y su reconciliacion al abrir (RFC-0010, E2c-1) · §568: 137 -> 146, la vista de recibos (RFC-0010, E2c-2) · §569: 146 -> 155, el registro conectado: anota al recibir y se reconcilia al arrancar (RFC-0010, E2c-2) · §570: 155 -> 160, el nodo compone, firma y sirve la cabeza v6 (RFC-0010, E2d) · §571: 160 -> 165, el recibo de recepcion en el cable y su camino (RFC-0010, E3) · §573: 165 -> 166, el arbol de recibos atado a RECEP_DEPTH (RFC-0010, E4) · §576: 166 -> 167, el alcance del recibo, decidido y atado (RFC-0010, D-E) · §594: 167 -> 171, el gate del diario en todo estado que lea un contador (ECST §8.1): dos positivos, dos rojos · §611: 171 -> 173, el lote y la prenda evaluada llevan su recibo, y el testigo del §576 cae al reves (RFC-0014, E3) · §636: 173 -> 175, la firma a demanda que no existe y el diario que si dice de que clave es cada linea, atados · §638: 175 -> 176, la clave nueva que sigue la cuenta de la vieja (el alcance de lo que dijo el §636) · §650: 176 -> 177, un hex con un multibyte es -32602 y el nodo sigue, sin PARADA · §655: 177 -> 179, un fallo del almacen pone el nodo en PARADA, y un rechazo de otra causa no (SEG-06 c) · §666: 179 -> 180, un byte ilegible en el diario se lleva su linea y no el diario · §667: 180 -> 181, con las hojas del MMR envenenadas no hay pareja ni latido · §672: 181 -> 182, el arbol de la epoca con rebuild_from, la misma raiz que hoja a hoja · §673: 182 -> 183, la frontera del MMR da la cima de todas las hojas · §674: 183 -> 184, el indice del diario da lo mismo que leerlo entero · §644: 184 -> 188, el nodo firma sus actas: la decision al arrancar, el arranque entero con claves de verdad, el aviso de agotamiento y la linea del acta que ningun lector de cabezas ve (RFC-0015, E3a)
zk-ssl-verify      sello      172   0   0   120  alias=verificador independiente · §257: 22 -> 23, del cable a la cabeza sin la capa · §291: 39 -> 46, el MMR de cabezas · §292: 46 -> 48, la cima en el digest · §297: 48 -> 55, el objeto de la cofirma · §322: 55 -> 61, las cofirmas dentro del paquete de evidencia · §324: 61 -> 62, la mudanza de COFIRMA_VERSION · §332: 62 -> 66, el indice declarado se ata al que va dentro de la firma · §335: 66 -> 69, el apano del OID pasa a tener un solo dueno · S395: 69 -> 72, la autonomia del verificador deja de ser prosa · §399: 72 -> 75, el indice declarado de la cabeza se ata al que va dentro de la firma · §406: 75 -> 79, el conjunto de versiones tiene un solo productor · §414: 79 -> 82, la cabeza v4 en el nucleo y el mando (RFC-0006, E2a) · §419: 82 -> 88, las reglas del arbol de consumos en el verificador (RFC-0006, E3b-2) · §430: 88 -> 92, el lector del conflicto entre libros (RFC-0006, E4a-2b) · §451: 92 -> 96, la cabeza v5 en el nucleo y el mando (RFC-0007, E1a) · §458: 96 -> 100, las reglas del arbol de congelados (RFC-0007, E3b) · §465: 100 -> 104, el kit verifica la prueba de edad sin el probador (RFC-0007, E4b-2) · §475: 104 -> 108, las reglas del arbol de cuentas y el noveno brazo del mando (RFC-0007, E5, 3b) · §495: 108 -> 112, el brazo del cobro pendiente en el mando (RFC-0008, E1) · §506: 112 -> 116, el brazo del pago en curso en el mando (RFC-0008, E2) · §520: 116 -> 120, el octavo brazo del mando, `tipo: "prenda"` (RFC-0008, E3) · §558: 120 -> 121, la variante V6 entra en el conjunto y los sobres preguntan por el predicado (RFC-0010, E2a) · §562: 121 -> 129, las reglas del arbol de recibos de recepcion · §566: 129 -> 132, el acuse y la inclusion contra la cabeza v6 (RFC-0010, E2a) · §567: 132 -> 133, el rx que no pertenece no tiene posicion: la era (Q, R] (RFC-0010, E2b) · §573: 133 -> 139, el sobre de completitud: sus veredictos, el cuarto estado y el camino del recibo (RFC-0010, E4) · §586: 139 -> 140, el ancho del indice sale del conjunto de xmss (entrada 101) · §592: 140 -> 144, el sobre del ancla: sus tres modos y la vista dividida (RFC-0012, E3) · §612: 144 -> 146, la resolucion del lote: su composicion y su forma juzgada otra vez (RFC-0014, E4a) · §613: 146 -> 147, la resolucion de la prenda se ata por su prueba y lleva una de tres (RFC-0014, E4b) · §633: 147 -> 152, el sobre del ancla cofirmada (RFC-0013, E4a): la forma, una cabeza firmada de verdad con su ancla en el medio, cada mentira por su nombre -el medio ajeno bien firmado incluido-, el desconocido que lo enumera y el kit que verifica el medio sin firmar · §640: 152 -> 157, el mando lee canonico -un u64 y un elemento menores que p- y la particion del MMR es total (RFC-0016) · §650: 157 -> 160, el kit lee el hex canonico: un multibyte es ROJO y no 101 · §651: 160 -> 165, las cinco familias del kit con el meta vaciado son ROJO, no 101 (D3-META) · §653: 165 -> 166, una prueba con bytes de cola es ROJO (SEG-04) · §662: 166 -> 167, cada u64 del sobre en su escritura minima · §664: 167 -> 168, una clave publicada con el OID ya apanado no se lee · §643: 168 -> 172, el acta de clave: la genesis, la rotacion comprometida y un rojo por regla (RFC-0015, E2)
zk-ssl-hash        sello      52   0   0    60  §270: 14 -> 16, el acuse componible por un tercero · §292: 22 -> 24, la cima en el digest · §414: 24 -> 26, epoch_digest_v4 (RFC-0006, E2a) · §416: 26 -> 27, la posicion del consumo en el nucleo (RFC-0006, E3a) · §451: 27 -> 31, epoch_digest_v5 y params_digest (RFC-0007, E1a) · §557: 31 -> 34, epoch_digest_v6, recibo_digest y el dominio RECEP_V1 (RFC-0010, E2a-0) · §591: 34 -> 37, ancla_digest, huella_de_clave y el dominio ANCLA_V1 (RFC-0012, E2) · §610: 37 -> 40, la huella del lote, con dominio propio y longitud codificada (RFC-0014, E2) · §640: 40 -> 45, la canonicidad del nucleo: el modulo del campo, la lectura canonica y la biyeccion de la escritura (RFC-0016) · §650: 45 -> 49, hex_canonico y bytes_de_hex sobre bytes, y la puerta que no deja trocear un &str · §662: 49 -> 50, la QUANTITY en su escritura minima: cantidad_canonica · §643: 50 -> 52, acta_digest y el dominio ACTAS_V1 (RFC-0015, E2)
zk-ssl-guardian    sello       28   0   0    60  §296: el guardian del indice, MUDADO del nodo (sus 9 tests salen del pin 82) · §298: 9 -> 11, y llega con sus dos tests de layout · §330: 11 -> 19, el lector comun de la semilla · §331: 19 -> 22, ClaveEnCero y su guarda · §335: 22 -> 25, el escritor del indice y su techo derivado del ancho · §336: 25 -> 26, el invariante del arranque tiene dueno · §594: 26 -> 27, un contador borrado reabre en cero y COINCIDE con la clave en cero: la premisa del ECST §8.1, medida · §650: 27 -> 28, la semilla se lee sobre bytes: un multibyte o un + se rechazan sin panico
zk-ssl-medio       sello      37   0   0    60  §631: nace con 25, el arbol del medio del ancla -SHA-256 de RFC 6962, copiado de mtc-core d3b0ca6 con su procedencia-: los 65.058 casos acumulados del IETF, sus vectores grandes hasta 2^64-1 hojas, las 685 sondas de transparency-dev/merkle (43 positivas, 4 de ellas sin tipo; 642 negativas, ninguna aceptada) y 46 negativas de 32 bytes hechas de sus positivas, que tapan el punto ciego medido: sin comparar la raiz vieja se pasaban las 685 (RFC-0013, E2a). 6 s en frio · §632: 25 -> 37, la nota del medio (RFC-0013, E2b): la base64 estricta de mtc-core, la nota checkpoint firmada con ML-DSA-44 tipo 0x06 y 33 notas contrastadas con torchwood v0.10.0 sobre el ML-DSA de Go 1.27 -la linea del publicador sale byte a byte, ninguna negativa que torchwood rechace la acepta el medio, y 4 que torchwood acepta caen aqui por tres reglas declaradas-; 11 s en frio con ml-dsa
zk-ssl-air         sello      42   0   0    60  §463: nace con 4, el AIR de la prueba de edad y su juez, sin el probador (RFC-0007, E4b-1) · §465: 4 -> 5, la m de una marca es la minima (RFC-0007, E4b-2) · §477: 5 -> 10, el AIR de la banda del saldo, sin titularidad (RFC-0007, E5, corte 4a) · §490: 10 -> 15, el AIR del cobro pendiente, sin el probador (RFC-0008, E1) · §495: 15 -> 16, el enlace del cobro pendiente a la cabeza v5 (RFC-0008, E1) · §503: 16 -> 22, el AIR del pago en curso y sus seis testigos (RFC-0008, E2) · §516: 22 -> 29, el AIR de la prenda con su juez y su enlace, sin el probador (RFC-0008, E3) · §535: 29 -> 36, la sal como tipo -MerkleConSal y sus siete testigos-, sin encender nada (RFC-0009, E3b-0) · §575: 36 -> 40, la lectura de una prueba no reserva lo que sus bytes no traen: el lote con sal y Proof::from_bytes · §578: 40 -> 41, toda verificacion del kit lee con sal: las cinco · §659: 41 -> 42, un nodo de mas en el lote no verifica: cada vector trae los nodos que la raiz consume (SEG-03b)
winter-air         sello      49   0   0    60  FORK de winterfell 0.13.1, APAGADO (RFC-0009 E3a-1, §533): los 49 tests de upstream, tal cual; entra por [patch.crates-io] con su nombre, y la foto de los circuitos (394) lo vigila · §575: Proof::from_bytes lee con un lector ACOTADO (src/proof/acotado.rs, declarado en su README); los 49 de upstream, tal cual
winter-prover      sello       6   0   0    60  FORK de winterfell 0.13.1, APAGADO (§533): los 6 tests de upstream; sin el bench de criterion; el lint de trace_table.rs:199 arreglado, por eso 0 warnings
winter-verifier    sello       0   0   0    60  FORK de winterfell 0.13.1, APAGADO (§533): upstream no trae tests; 0 es CORRECTO
settlement-prover  sello       0   0   0   300  sin tests
nova-experiment    sello       0   0   0   300  0 es CORRECTO: sus 3 tests exigen --features test-setup
halo2-experiment   largo      27   0   0  1200  caro: 438 s medidos
plonk-experiment   largo      36   0   3  1800  caro: 749 s medidos. DEUDA: 3 warnings
zk-core            completo   74   0  10  3600  2075 s medidos en 6cb8883 (§260). La nota anterior (§225) decia 38,6 min = 1472 lib + 845 ceremonia: envejecio A LA BAJA. Los 10 warnings son de `ceremony`, no suyos
FIN_TABLA
)

# ── lo que cubre el nivel --completo, y nadie mas (§284) ────────
# La regla vieja «tocar un Cargo.* vence la foto» era SINTACTICA donde el
# riesgo es SEMANTICO: sobredisparaba sobre todo el proyecto salvo esto.
# `--completo` añade UNA fila sobre --sello: zk-core (su fila, arriba).
# Cierre real, leido de crates/zk-core/Cargo.toml: settlement-prover
# (path), ceremony (dev-dependency a proposito, grafo aciclico) y los
# ark-* que pincha el [workspace.dependencies] del Cargo.toml RAIZ.
#   - VENCE la foto: un cambio bajo estas rutas, o lineas ark-* en el
#     diff del Cargo.toml raiz.
#   - NO la vence (declarado ANTES de mirar nada): el Cargo.lock solo
#     —un lock movido sin toml cubierto imprime que VALE, con la razon—
#     ni la raiz tocada sin ark-*.
COMPLETO_CUBRE="crates/zk-core crates/settlement-prover crates/ceremony"

# ── utilidades ──────────────────────────────────────────────────
rojo=0
fallos=()
msg() { echo "$*" >&2; }
falla() { rojo=1; fallos+=("$1"); msg "  XX  $1"; }

nivel_num() {
  case "$1" in
    sello) echo 1 ;; largo) echo 2 ;; completo) echo 3 ;; *) echo 9 ;;
  esac
}

miembros_del_workspace() {
  sed -n '/^\[workspace\]/,/^\[[^w]/p' Cargo.toml \
    | grep -oE '"crates/[a-z0-9-]+"' | tr -d '"' | sed 's|crates/||'
}

estado_completo() {
  if [ -f "$SELLO_FILE" ]; then
    local c f n toc raiz lock ark
    read -r c f < "$SELLO_FILE"
    n=$(git rev-list --count "$c..HEAD" 2>/dev/null || echo "?")
    if [ "$n" = "0" ]; then
      msg "  ultimo --completo: $c ($f) · AL DIA"
    else
      msg "  ultimo --completo: $c ($f) · **$n sello(s) por detras de HEAD**"
      if [ "$n" = "?" ]; then
        msg "  la foto apunta a $c y ese sello ya no se alcanza: se remide con --completo"
      else
        toc=$(git diff --name-only "$c..HEAD" -- $COMPLETO_CUBRE 2>/dev/null)
        raiz=$(git diff --name-only "$c..HEAD" -- Cargo.toml 2>/dev/null)
        lock=$(git diff --name-only "$c..HEAD" -- Cargo.lock 2>/dev/null)
        ark=0
        [ -n "$raiz" ] && ark=$(git diff "$c..HEAD" -- Cargo.toml 2>/dev/null | grep -cE '^[+-].*ark-')
        if [ -n "$toc" ]; then
          msg "  ⚠️ VENCIDA: lo cubierto cambio desde la foto — toca --completo:"
          sed 's/^/      /' <<< "$toc" >&2
        elif [ "${ark:-0}" -gt 0 ]; then
          msg "  ⚠️ VENCIDA: el Cargo.toml raiz movio lineas ark-* ($ark) — toca --completo"
        elif [ -n "$raiz" ]; then
          msg "  la raiz cambio sin mover ark-*: por detras pero limpio en lo cubierto, la foto VALE"
        elif [ -n "$lock" ]; then
          msg "  el lock se movio sin un Cargo.toml cubierto: por detras pero limpio en lo cubierto, la foto VALE"
        else
          msg "  por detras pero limpio en lo cubierto: la foto VALE"
        fi
      fi
    fi
  else
    msg "  ultimo --completo: **NUNCA**. Los 54 min del nivel completo no los ha corrido nadie."
  fi
}

# Los bancos (§582): cuando pasaron VERDES por ultima vez, y si cambio lo que ejercen.
estado_bancos() {
  if [ -f "$BANCOS_FILE" ]; then
    local c f k n toc
    read -r c f k < "$BANCOS_FILE"
    n=$(git rev-list --count "$c..HEAD" 2>/dev/null || echo "?")
    if [ "$n" = "0" ]; then
      msg "  ultimo --bancos: $c ($f, $k bancos) · AL DIA"
    elif [ "$n" = "?" ]; then
      msg "  ultimo --bancos: $c ($f) · ese sello ya no se alcanza: se remide con --bancos"
    else
      toc=$(git diff --name-only "$c..HEAD" -- crates Cargo.toml Cargo.lock 'tools/banco_*' \
            2>/dev/null | wc -l | tr -d ' ')
      if [ "$toc" != "0" ]; then
        msg "  ultimo --bancos: $c ($f) · **$n sello(s) por detras**, y $toc fichero(s) de lo que ejercen cambiados — toca --bancos"
      else
        msg "  ultimo --bancos: $c ($f) · $n sello(s) por detras, limpio en lo que ejercen: la foto VALE"
      fi
    fi
  else
    msg "  ultimo --bancos: **NUNCA** desde que se anotan (§582). Los ~15 min de --bancos no los ha corrido nadie."
  fi
}

case "$NIVEL" in
  --sello|--largo|--completo|--lista|--bancos) : ;;
  *) msg "nivel desconocido: $NIVEL. Usa --sello, --largo, --completo, --bancos o --lista."; exit 2 ;;
esac

# ── 0 · COHERENCIA: la tabla y el workspace dicen lo mismo ──────
msg "== CANON · coherencia con el workspace =="
MIEMBROS=$(miembros_del_workspace)
EN_TABLA=$(awk 'NF{print $1}' <<< "$TABLA")
n_m=$(wc -l <<< "$MIEMBROS"); n_t=$(wc -l <<< "$EN_TABLA")
msg "  miembros en Cargo.toml: $n_m · filas en la tabla: $n_t"
# ⚠️ Here-string, NO tuberia: ver el punto 6 de la cabecera.
for c in $MIEMBROS; do
  grep -qx "$c" <<< "$EN_TABLA" || falla "el crate '$c' esta en el workspace y NO en el canon"
done
for c in $EN_TABLA; do
  grep -qx "$c" <<< "$MIEMBROS" || falla "la fila '$c' no corresponde a ningun crate del workspace"
done
[ $rojo -eq 0 ] && msg "  OK  todos los crates tienen fila y todas las filas tienen crate"
estado_completo
estado_bancos

if [ "$NIVEL" = "--bancos" ]; then
  msg ""
  bash tools/bancos.sh; rb=$?
  [ $rojo -eq 0 ] || rb=1
  exit $rb
fi

if [ "$NIVEL" = "--lista" ]; then
  msg ""
  msg "== LA TABLA =="
  sed 's/^/  /' <<< "$TABLA" >&2
  exit $rojo
fi

# ── 1 · los tests, crate a crate ────────────────────────────────
PEDIDO=$(nivel_num "${NIVEL#--}")
OUT="${OUT:-/tmp/canon}"
rm -rf "$OUT"; mkdir -p "$OUT"
msg ""
msg "== CANON $NIVEL · tests =="
msg "   crate                exit  pasan(pin) ignor(pin) warn(pin)   seg"
T_TOTAL=0
while read -r c niv pasan ign warn tmo resto; do
  [ -n "$c" ] || continue
  [ "$(nivel_num "$niv")" -le "$PEDIDO" ] || continue
  T0=$(date +%s)
  timeout "${tmo}s" cargo test -p "$c" --release > "$OUT/$c.txt" 2>&1
  RC=$?
  T1=$(date +%s); T_TOTAL=$((T_TOTAL + T1 - T0))
  P=$(grep -oE "[0-9]+ passed" "$OUT/$c.txt" | grep -oE "[0-9]+" | paste -sd+ | bc 2>/dev/null); P=${P:-0}
  I=$(grep -oE "[0-9]+ ignored" "$OUT/$c.txt" | grep -oE "[0-9]+" | paste -sd+ | bc 2>/dev/null); I=${I:-0}
  W=$(grep -c "^warning" "$OUT/$c.txt")
  printf "   %-20s %4s %6s(%s) %6s(%s) %5s(%s) %5s\n" "$c" "$RC" "$P" "$pasan" "$I" "$ign" "$W" "$warn" "$((T1-T0))" >&2
  [ "$RC" = "0" ]     || falla "$c: exit $RC  ->  tools/canon.sh, fila '$c'"
  [ "$P" = "$pasan" ] || falla "$c: pasan $P, el canon dice $pasan  ->  MEDIR y editar tools/canon.sh, fila '$c'"
  [ "$I" = "$ign" ]   || falla "$c: ignorados $I, el canon dice $ign  ->  tools/canon.sh, fila '$c'"
  [ "$W" = "$warn" ]  || falla "$c: warnings $W, el canon dice $warn  ->  tools/canon.sh, fila '$c'"
done <<< "$TABLA"
msg "   ── total: ${T_TOTAL} s"

# ── 2 · tests que no protegen ───────────────────────────────────
msg ""
msg "== CANON · tests que no protegen =="
python3 tools/check_tests.py > "$OUT/check_tests.txt" 2>&1
if [ $? -eq 0 ]; then
  msg "  OK  $(tail -2 "$OUT/check_tests.txt" | tr '\n' ' ')"
else
  sed 's/^/      /' "$OUT/check_tests.txt" >&2
  falla "check_tests.py encontro tests que no protegen"
fi

# ── 2 bis · ficheros .rs que no declara nadie ──
# Un `.rs` sin declarar NO se compila, asi que nada lo verifica — y encima
# invita a especular sobre el. `marlin_proof_system.rs` estuvo asi, y se
# sello DOS VECES una descripcion falsa de lo que era (§227).
msg ""
msg "== CANON · ficheros .rs que no declara nadie =="
python3 tools/check_modulos.py > "$OUT/check_modulos.txt" 2>&1
if [ $? -eq 0 ]; then
  msg "  OK  $(tail -1 "$OUT/check_modulos.txt")"
else
  sed 's/^/      /' "$OUT/check_modulos.txt" >&2
  falla "hay ficheros .rs que nadie declara"
fi

# ── 2 ter · las herramientas del bucle ──────────────────────────
# Hasta 269 el canon invocaba DOS de las siete de `tools/`. Las otras cinco
# las corria cada BLOQUE a mano, y una herramienta que nadie recuerda meter
# en un bloque es una herramienta que nadie corre: DOS llevaban rojas sin
# que nadie lo viera. Es exactamente lo que le paso a `check_tests.py`, y
# por lo que esta aqui arriba.
#
# Entran a PIN CERO. No hay pin de fallos: los rojos que tenian eran defecto
# de las propias herramientas —exclusiones que les faltaban y un patron que
# tomaba una plantilla por cita—, no deuda de documentacion, y 269 los
# arreglo ANTES de cablearlas.
msg ""
msg "== CANON · las herramientas de tools/ =="
for H in check_cifras check_figures check_columns check_constraint_layout verificar_citas check_dominios check_publicadas check_nucleo check_techo; do
  python3 "tools/$H.py" > "$OUT/$H.txt" 2>&1
  if [ $? -eq 0 ]; then
    msg "  OK  $H"
  else
    sed 's/^/      /' "$OUT/$H.txt" >&2
    falla "$H.py en rojo"
  fi
done

# ── 3 · conformidad ─────────────────────────────────────────────
msg ""
msg "== CANON · conformidad =="
cargo run --release -p zk-ssl-cli -- conformance --check spec/vectors/zkssl-0.4.json > "$OUT/c04.txt" 2>&1
if grep -q "todo IDENTICO" "$OUT/c04.txt"; then msg "  OK  0.4 -> todo IDENTICO (en lo que el circuito fija, D-AI)"; else falla "0.4 NO da 'todo IDENTICO'"; fi
cargo run --release -p zk-ssl-cli -- conformance --check spec/vectors/zkssl-0.3.json > "$OUT/c03.txt" 2>&1
if grep -q "OTRA version" "$OUT/c03.txt"; then msg "  OK  0.3 RECHAZADO (otra version)"; else falla "0.3 deberia rechazarse por version"; fi
cargo run --release -p zk-ssl-cli -- conformance --check spec/vectors/zkssl-0.2.json > "$OUT/c02.txt" 2>&1
if grep -q "OTRA version" "$OUT/c02.txt"; then msg "  OK  0.2 RECHAZADO (otra version)"; else falla "0.2 deberia rechazarse por version"; fi
cargo run --release -p zk-ssl-cli -- conformance --check spec/vectors/zkssl-0.1.json > "$OUT/c01.txt" 2>&1
if grep -q "OTRA version" "$OUT/c01.txt"; then msg "  OK  0.1 RECHAZADO (otra version)"; else falla "0.1 deberia rechazarse por version"; fi

# ── 3 bis · conformidad del PAQUETE (spec/PAQUETE.md, RFC-0004 E2, §398; el arnes: RFC-0005 E4, §408) ──
msg ""
msg "== CANON · conformidad del paquete =="
cargo build --release -q -p zk-ssl-verify > "$OUT/paquete_build.txt" 2>&1 || falla "zk-ssl-verify no compila en release"
# UN solo productor del bucle del manifiesto: tools/conformidad.sh, el mismo que un tercero corre
# sobre cualquier binario y el que artefacto.sh corre sobre el binario con remap. Cada ROJO del
# arnes entra aqui con su nombre; la prueba de vida (manifiesto vacio, vector sin entrada) es suya.
if bash tools/conformidad.sh target/release/zk-ssl-verify > "$OUT/paquete.txt" 2>&1; then
  msg "  OK  paquete: $(tail -n 1 "$OUT/paquete.txt" | sed 's/^conformidad: //')"
else
  while IFS= read -r L; do falla "paquete $L"; done < <(grep '^ROJO' "$OUT/paquete.txt" | sed 's/^ROJO //')
  grep -q '^ROJO' "$OUT/paquete.txt" || falla "paquete: el arnes falla sin nombrar la entrada ($(tail -n 1 "$OUT/paquete.txt"))"
fi

# ── 3 bis cable · los rechazos del CABLE (RFC-0005 E3, §409): el MISMO arnes, otro manifiesto ──
msg ""
msg "== CANON · los rechazos del cable =="
cargo build --release -q -p zk-ssl-cli > "$OUT/cable_build.txt" 2>&1 || falla "zk-ssl-cli no compila en release"
# tools/cable_respuesta.sh es el adaptador al contrato del mando: un argumento, exit 0/1/2; una
# segunda implementacion del consumidor pone su ejecutable en su lugar. Cada ROJO entra por falla.
if bash tools/conformidad.sh tools/cable_respuesta.sh spec/vectors/cable/MANIFIESTO.txt > "$OUT/cable.txt" 2>&1; then
  msg "  OK  cable: $(tail -n 1 "$OUT/cable.txt" | sed 's/^conformidad: //')"
else
  while IFS= read -r L; do falla "cable $L"; done < <(grep '^ROJO' "$OUT/cable.txt" | sed 's/^ROJO //')
  grep -q '^ROJO' "$OUT/cable.txt" || falla "cable: el arnes falla sin nombrar la entrada ($(tail -n 1 "$OUT/cable.txt"))"
fi

# ── 3 bis consumo · el sobre de CONSUMO (RFC-0006 E3, §422): el MISMO arnes, otro manifiesto ──
msg ""
msg "== CANON · los rechazos del sobre de consumo =="
# El binario es el MISMO que el 3 bis ya construyo en release: no se vuelve a compilar.
# Un solo productor del bucle, tools/conformidad.sh, con OTRO manifiesto. Cada ROJO entra por falla.
if bash tools/conformidad.sh target/release/zk-ssl-verify spec/vectors/consumo/MANIFIESTO.txt > "$OUT/consumo.txt" 2>&1; then
  msg "  OK  consumo: $(tail -n 1 "$OUT/consumo.txt" | sed 's/^conformidad: //')"
else
  while IFS= read -r L; do falla "consumo $L"; done < <(grep '^ROJO' "$OUT/consumo.txt" | sed 's/^ROJO //')
  grep -q '^ROJO' "$OUT/consumo.txt" || falla "consumo: el arnes falla sin nombrar la entrada ($(tail -n 1 "$OUT/consumo.txt"))"
fi

# ── 3 bis conflicto · el sobre de CONFLICTO (RFC-0006 E4a, §431): el MISMO arnes ──
msg ""
msg "== CANON · los rechazos del sobre de conflicto =="
# El binario es el MISMO que el 3 bis ya construyo en release: no se vuelve a compilar.
# Un solo productor del bucle, tools/conformidad.sh, con OTRO manifiesto. Cada ROJO entra por falla.
if bash tools/conformidad.sh target/release/zk-ssl-verify spec/vectors/conflicto/MANIFIESTO.txt > "$OUT/conflicto.txt" 2>&1; then
  msg "  OK  conflicto: $(tail -n 1 "$OUT/conflicto.txt" | sed 's/^conformidad: //')"
else
  while IFS= read -r L; do falla "conflicto $L"; done < <(grep '^ROJO' "$OUT/conflicto.txt" | sed 's/^ROJO //')
  grep -q '^ROJO' "$OUT/conflicto.txt" || falla "conflicto: el arnes falla sin nombrar la entrada ($(tail -n 1 "$OUT/conflicto.txt"))"
fi

# ── 3 bis rechazo · el sobre de RECHAZO (RFC-0007 E3, desde el §455): el MISMO arnes ──
msg ""
msg "== CANON · las causas del sobre de rechazo =="
# El binario es el MISMO que el 3 bis ya construyo en release: no se vuelve a compilar.
# Un solo productor del bucle, tools/conformidad.sh, con OTRO manifiesto. Cada ROJO entra por falla.
if bash tools/conformidad.sh target/release/zk-ssl-verify spec/vectors/rechazo/MANIFIESTO.txt > "$OUT/rechazo.txt" 2>&1; then
  msg "  OK  rechazo: $(tail -n 1 "$OUT/rechazo.txt" | sed 's/^conformidad: //')"
else
  while IFS= read -r L; do falla "rechazo $L"; done < <(grep '^ROJO' "$OUT/rechazo.txt" | sed 's/^ROJO //')
  grep -q '^ROJO' "$OUT/rechazo.txt" || falla "rechazo: el arnes falla sin nombrar la entrada ($(tail -n 1 "$OUT/rechazo.txt"))"
fi

# ── 3 bis edad · el sobre de EDAD (RFC-0007 E4b-3, desde el §467): el MISMO arnes ──
msg ""
msg "== CANON · las formas del sobre de edad =="
# El binario es el MISMO que el 3 bis ya construyo en release: no se vuelve a compilar.
# Un solo productor del bucle, tools/conformidad.sh, con OTRO manifiesto. Cada ROJO entra por falla.
if bash tools/conformidad.sh target/release/zk-ssl-verify spec/vectors/edad/MANIFIESTO.txt > "$OUT/edad.txt" 2>&1; then
  msg "  OK  edad: $(tail -n 1 "$OUT/edad.txt" | sed 's/^conformidad: //')"
else
  while IFS= read -r L; do falla "edad $L"; done < <(grep '^ROJO' "$OUT/edad.txt" | sed 's/^ROJO //')
  grep -q '^ROJO' "$OUT/edad.txt" || falla "edad: el arnes falla sin nombrar la entrada ($(tail -n 1 "$OUT/edad.txt"))"
fi

# ── 3 bis pendiente · el sobre de COBRO PENDIENTE (RFC-0008 E4-cobro, desde el §499): el MISMO arnes ──
msg ""
msg "== CANON · las formas del sobre de cobro pendiente =="
# El binario es el MISMO que el 3 bis ya construyo en release: no se vuelve a compilar.
# Un solo productor del bucle, tools/conformidad.sh, con OTRO manifiesto. Cada ROJO entra por falla.
if bash tools/conformidad.sh target/release/zk-ssl-verify spec/vectors/pendiente/MANIFIESTO.txt > "$OUT/pendiente.txt" 2>&1; then
  msg "  OK  pendiente: $(tail -n 1 "$OUT/pendiente.txt" | sed 's/^conformidad: //')"
else
  while IFS= read -r L; do falla "pendiente $L"; done < <(grep '^ROJO' "$OUT/pendiente.txt" | sed 's/^ROJO //')
  grep -q '^ROJO' "$OUT/pendiente.txt" || falla "pendiente: el arnes falla sin nombrar la entrada ($(tail -n 1 "$OUT/pendiente.txt"))"
fi

# ── 3 bis pago · el sobre de PAGO EN CURSO (RFC-0008 E2, desde el §509): el MISMO arnes ──
msg ""
msg "== CANON · las formas del sobre de pago en curso =="
# El binario es el MISMO que el 3 bis ya construyo en release: no se vuelve a compilar.
# Un solo productor del bucle, tools/conformidad.sh, con OTRO manifiesto. Cada ROJO entra por falla.
if bash tools/conformidad.sh target/release/zk-ssl-verify spec/vectors/pago/MANIFIESTO.txt > "$OUT/pago.txt" 2>&1; then
  msg "  OK  pago: $(tail -n 1 "$OUT/pago.txt" | sed 's/^conformidad: //')"
else
  while IFS= read -r L; do falla "pago $L"; done < <(grep '^ROJO' "$OUT/pago.txt" | sed 's/^ROJO //')
  grep -q '^ROJO' "$OUT/pago.txt" || falla "pago: el arnes falla sin nombrar la entrada ($(tail -n 1 "$OUT/pago.txt"))"
fi

# ── 3 bis prenda · el sobre de PRENDA (RFC-0008 E3, desde el §546): el MISMO arnes ──
msg ""
msg "== CANON · las formas del sobre de prenda =="
# El binario es el MISMO que el 3 bis ya construyo en release: no se vuelve a compilar.
# Un solo productor del bucle, tools/conformidad.sh, con OTRO manifiesto. Cada ROJO entra por falla.
if bash tools/conformidad.sh target/release/zk-ssl-verify spec/vectors/prenda/MANIFIESTO.txt > "$OUT/prenda.txt" 2>&1; then
  msg "  OK  prenda: $(tail -n 1 "$OUT/prenda.txt" | sed 's/^conformidad: //')"
else
  while IFS= read -r L; do falla "prenda $L"; done < <(grep '^ROJO' "$OUT/prenda.txt" | sed 's/^ROJO //')
  grep -q '^ROJO' "$OUT/prenda.txt" || falla "prenda: el arnes falla sin nombrar la entrada ($(tail -n 1 "$OUT/prenda.txt"))"
fi

# ── 3 bis completitud · el sobre de COMPLETITUD (RFC-0010 E5, desde el §574): el MISMO arnes ──
msg ""
msg "== CANON · las formas del sobre de completitud =="
# El binario es el MISMO que el 3 bis ya construyo en release: no se vuelve a compilar.
# Un solo productor del bucle, tools/conformidad.sh, con OTRO manifiesto. Cada ROJO entra por falla.
if bash tools/conformidad.sh target/release/zk-ssl-verify spec/vectors/completitud/MANIFIESTO.txt > "$OUT/completitud.txt" 2>&1; then
  msg "  OK  completitud: $(tail -n 1 "$OUT/completitud.txt" | sed 's/^conformidad: //')"
else
  while IFS= read -r L; do falla "completitud $L"; done < <(grep '^ROJO' "$OUT/completitud.txt" | sed 's/^ROJO //')
  grep -q '^ROJO' "$OUT/completitud.txt" || falla "completitud: el arnes falla sin nombrar la entrada ($(tail -n 1 "$OUT/completitud.txt"))"
fi

# ── 3 bis ancla · el sobre del ANCLA (RFC-0012 E4, desde el §593): el MISMO arnes ──
msg ""
msg "== CANON · las formas del sobre del ancla =="
# El binario es el MISMO que el 3 bis ya construyo en release: no se vuelve a compilar.
# Un solo productor del bucle, tools/conformidad.sh, con OTRO manifiesto. Cada ROJO entra por falla.
if bash tools/conformidad.sh target/release/zk-ssl-verify spec/vectors/ancla/MANIFIESTO.txt > "$OUT/ancla.txt" 2>&1; then
  msg "  OK  ancla: $(tail -n 1 "$OUT/ancla.txt" | sed 's/^conformidad: //')"
else
  while IFS= read -r L; do falla "ancla $L"; done < <(grep '^ROJO' "$OUT/ancla.txt" | sed 's/^ROJO //')
  grep -q '^ROJO' "$OUT/ancla.txt" || falla "ancla: el arnes falla sin nombrar la entrada ($(tail -n 1 "$OUT/ancla.txt"))"
fi

# ── 3 bis ancla cofirmada · el sobre del ANCLA COFIRMADA (RFC-0013 E4b, desde el §634) ──
msg ""
msg "== CANON · las formas del sobre del ancla cofirmada =="
# El binario es el MISMO que el 3 bis ya construyo en release: no se vuelve a compilar.
# Un solo productor del bucle, tools/conformidad.sh, con OTRO manifiesto. Cada ROJO entra por falla.
if bash tools/conformidad.sh target/release/zk-ssl-verify spec/vectors/ancla-cofirmada/MANIFIESTO.txt > "$OUT/ancla_cofirmada.txt" 2>&1; then
  msg "  OK  ancla-cofirmada: $(tail -n 1 "$OUT/ancla_cofirmada.txt" | sed 's/^conformidad: //')"
else
  while IFS= read -r L; do falla "ancla-cofirmada $L"; done < <(grep '^ROJO' "$OUT/ancla_cofirmada.txt" | sed 's/^ROJO //')
  grep -q '^ROJO' "$OUT/ancla_cofirmada.txt" || falla "ancla-cofirmada: el arnes falla sin nombrar la entrada ($(tail -n 1 "$OUT/ancla_cofirmada.txt"))"
fi

# ── 3 duodecies · la SEGUNDA implementacion (BACKLOG 85; tools/segunda/) ──
# Otro codigo, en Python y sin dependencias, reproduce el nucleo congelado desde NUCLEO.md (26 KAT)
# y verifica las cabezas firmadas de TODOS los vectores con XMSS^MT escrito desde RFC 8391
# (tools/segunda/kat_xmss/xmss.py, dentro del directorio extraible). Esta
# aqui por lo mismo que las herramientas del bucle (2 ter): un KAT nuevo o una version nueva de
# cabeza que la segunda implementacion no reproduzca tiene que verse en el sello, no cuando alguien
# se acuerde de correrla. Pin cero: los tres jueces y el segundo verificador fallan con nombre. ~45 s,
# casi todos de XMSS en Python.
msg ""
msg "== CANON · la segunda implementacion (tools/segunda) =="
T_SEG0=$(date +%s)
if python3 tools/segunda/juez_nucleo.py > "$OUT/segunda_nucleo.txt" 2>&1; then
  msg "  OK  $(tail -n 1 "$OUT/segunda_nucleo.txt")"
else
  grep '^ROJO' "$OUT/segunda_nucleo.txt" | sed 's/^/      /' >&2
  falla "la segunda implementacion NO reproduce los KAT del nucleo ($(tail -n 1 "$OUT/segunda_nucleo.txt"))"
fi
if python3 tools/segunda/juez_cabezas.py > "$OUT/segunda_cabezas.txt" 2>&1; then
  msg "  OK  $(tail -n 1 "$OUT/segunda_cabezas.txt") ($(( $(date +%s) - T_SEG0 )) s)"
else
  grep '^NO' "$OUT/segunda_cabezas.txt" | grep -v '\[negativo\]' | sed 's/^/      /' >&2
  falla "la segunda implementacion NO verifica las cabezas firmadas ($(tail -n 1 "$OUT/segunda_cabezas.txt"))"
fi
# El corpus KAT de XMSS^MT (tools/segunda/kat_xmss, entrada 77): firmas del crate clavado, juzgadas
# por la implementacion de RFC 8391. Si `xmss` cambia de version y de bytes, aqui se ve. ~1 s.
if python3 tools/segunda/kat_xmss/juez_xmss.py > "$OUT/segunda_kat_xmss.txt" 2>&1; then
  msg "  OK  $(tail -n 1 "$OUT/segunda_kat_xmss.txt" | sed 's/ - generador:.*//')"
else
  grep '^ROJO' "$OUT/segunda_kat_xmss.txt" | sed 's/^/      /' >&2
  falla "el corpus KAT de xmss NO verifica con la implementacion de RFC 8391 ($(tail -n 1 "$OUT/segunda_kat_xmss.txt"))"
fi
# El SEGUNDO VERIFICADOR (tools/segunda/verificador.py): las cinco formas sin STARK del paquete, con
# el contrato del mando, juzgadas por el MISMO arnes y los MISMOS manifiestos que el binario de
# referencia. Si un vector nuevo entra en estas familias, los dos codigos tienen que decir lo mismo.
for F in paquete consumo conflicto ancla; do
  if bash tools/conformidad.sh tools/segunda/verificador.py "spec/vectors/$F/MANIFIESTO.txt" > "$OUT/segunda_$F.txt" 2>&1; then
    msg "  OK  segunda $F: $(tail -n 1 "$OUT/segunda_$F.txt" | sed 's/^conformidad: //; s/ - binario.*//')"
  else
    while IFS= read -r L; do falla "segunda $F $L"; done < <(grep '^ROJO' "$OUT/segunda_$F.txt" | sed 's/^ROJO //')
    grep -q '^ROJO' "$OUT/segunda_$F.txt" || falla "segunda $F: el arnes falla sin nombrar la entrada ($(tail -n 1 "$OUT/segunda_$F.txt"))"
  fi
done
# El VERIFICADOR STARK (tools/segunda/stark.py y airs.py, cuarto hito de la 85, §626): las pruebas
# ocultas y con sal de las familias que el kit verifica hoy -rechazo, edad, pago, pendiente,
# prenda y completitud-, cada una con el enunciado que el mando le compone, contra lo que fija su
# MANIFIESTO, y tres falsadores por positivo. Las AIR estan transcritas del .rs: lo que este juez
# mide es la maquinaria (Fiat-Shamir, FRI, DEEP, Merkle con sal, Oculta), no las AIR. ~15 s.
if python3 tools/segunda/juez_stark.py > "$OUT/segunda_stark.txt" 2>&1; then
  msg "  OK  $(tail -n 1 "$OUT/segunda_stark.txt")"
else
  grep '^FALLA' "$OUT/segunda_stark.txt" | sed 's/^/      /' >&2
  falla "la segunda implementacion NO juzga las pruebas STARK como el manifiesto ($(tail -n 1 "$OUT/segunda_stark.txt"))"
fi

# ── 3 ter · el ARTEFACTO (tools/artefacto.sh --check, §401): la PROPIEDAD, no un pin ──
msg ""
msg "== CANON · el artefacto =="
T_ART0=$(date +%s)
if bash tools/artefacto.sh --check > "$OUT/artefacto.txt" 2>&1; then
  msg "  OK  $(tail -n 1 "$OUT/artefacto.txt" | sed 's/^ *OK *//') ($(( $(date +%s) - T_ART0 )) s)"
else
  tail -n 5 "$OUT/artefacto.txt" >&2; falla "el artefacto no es reproducible o no pasa el manifiesto"
fi

# ── veredicto ───────────────────────────────────────────────────
msg ""
if [ $rojo -eq 0 ]; then
  msg "== CANON $NIVEL: VERDE =="
  if [ "$NIVEL" = "--completo" ]; then
    mkdir -p "$(dirname "$SELLO_FILE")"
    printf '%s %s\n' "$(git rev-parse --short HEAD)" "$(date -Iseconds)" > "$SELLO_FILE"
    msg "   anotado en $SELLO_FILE — para que nadie tenga que acordarse."
  fi
else
  msg "== CANON $NIVEL: ROJO · ${#fallos[@]} fallo(s) =="
  for f in "${fallos[@]}"; do msg "   · $f"; done
fi
msg "   salida integra en $OUT/"
exit $rojo
