#!/usr/bin/env python3
"""ATADO B - la CONSTANTE contra los DOCUMENTOS.

Su gemelo es el test la_cifra_publicada_sigue_siendo_la_medida, en
crates/zk-ssl/src/metrics.rs, que ata el INSTRUMENTO a la constante.
Aqui se ata la constante a lo PUBLICADO. Cada rojo dice cual de las dos
cosas se quedo atras.

Por que existe: el 31-07-2026 el asiento §83.3 midio la deriva de la
cifra publicada en 2,2 % y la declaro "dentro de la guarda". El 14-08
era 4,8 %. Una TOLERANCIA declarada una vez sobre algo que crece es una
promesa, no un gate.

LAS DOS UNIDADES. La cifra es binaria. Un documento puede escribirla en
MiB (2^20) o dar su equivalente SI en MB (10^6); las dos son correctas y
las dos se derivan de los MISMOS bytes: PUBLICADA_PAGO_MIN_B y
PUBLICADA_PAGO_MAX_B, los extremos de la banda (S538). Lo que NO vale es
el valor binario con etiqueta MB, que es el defecto que §83.3 cazo y que
hace leer un 9,9 % menos.

VENTANA. El contexto se busca en la linea y en las DOS de alrededor: la
primera version miraba solo la linea y por eso no vio tres cifras SI que
viven un renglon mas abajo que su frase. Una frase que cruza un salto de
linea no se cuenta con un patron de una linea.

CITAS HISTORICAS. Una cifra que aparece junto a la marca de una version
anterior NO es lo que este repo publica: es lo que publico otro deposito.
Cambiarla seria falsificar el registro. Se salta, y **se imprime**: una
exclusion silenciosa es un agujero.

SIN DECIMAL (§308). Hasta aqui el patron EXIGIA parte decimal, y por eso
no veia siete citas vivas escritas con el entero pelado: 59 MB y 124 MiB.
Ahora el decimal es OPCIONAL. Y el patron ancla su borde IZQUIERDO: sin
eso "1.126,2 MiB" casaria como "126,2" -el separador de miles, el mismo
caso que ya mordio con un patron de 994 sobre 41.994-, y ese agujero lo
tenia la version anterior sin saberlo.

LA VIA (S362). La cifra se mide sobre `send`/`claim` de la capa, que es
la via que las cifras publicadas DESCRIBEN, y no sobre la del cliente.
Es deliberado y su permiso vive en `mod tests` de metrics.rs: cuando
llegue la migracion -- ENTRADA 32 del BACKLOG -- las mediciones cambian
con ella, y eso se DECLARA, no se absorbe.

MEDIDO el 2026-08-26: la via DOCUMENTADA -- send_materials ->
client::prove_send -> apply_send -- da los MISMOS bytes, 66_739 y
66_692, en cinco repeticiones y sin una sola diferencia. La cifra
publicada NO depende de la via. Desde el S362 lo PINA un segundo
gemelo, los_dos_lados_del_pago_atan_la_banda, que asierta esos dos
bytes ademas de la relacion temporal. ⚠️ §860: la relacion temporal ya no: el §538 (RFC-0009 D-AJ) retiro su aserto, y el test solo ata la banda de bytes de los dos lados; los tiempos se imprimen.

Ceguera declarada:
  - no ve una cifra escrita con palabras;
  - no entra en AUDITORIA.md: los asientos son REGISTRO HISTORICO y una
    cifra vieja dentro de un asiento es correcta;
  - no entra en doc/preprints/: entrada 28, SUSPENDIDA hasta el fin del
    proyecto. Lo que alli quede rancio se DECLARA, no se repara;
  - no vigila TIEMPOS ni RATIOS: dependen de la maquina y publicarlos en
    absoluto es una decision de diseno, no un numero que atar. Declarado
    en el asiento del §304.
  - no ve GB ni GiB, ni ninguna escala que no sea POR MIL: la CIFRA solo
    casa MiB|MB y los dos valores esperados salen de PAGO_B x 1000. Una
    cifra derivada al millon queda fuera del gate por construccion;
  - el CONTEXTO es un vocabulario fijo de frases "por mil", asi que una
    tabla que dice "1M op/dia" no lo dispara y sus cifras no se miran;
  - no entra en el CODIGO: solo recorre los .md de la raiz. log.rs:191
    publicaba la aritmetica de un paso y ningun gate la veia; se reparo a
    mano en §308, y extender el atado a los .rs queda por censar.

ATADO C (S403) - la URL DEL REPOSITORIO contra los documentos. Mismo
principio, otra constante: el repositorio se renombro y la URL vieja solo
vivia por una redireccion ajena; dieciseis lineas de diez ficheros la citaban
y ningun gate lo veia. URL_REPO se declara UNA vez aqui; el gate exige que
el token viejo no aparezca en ningun documento vivo ni en CITATION.cff, que
CITATION.cff declare exactamente URL_REPO en repository-code, y -prueba de
vida- que la URL nueva se cite al menos una vez: un universo vacio no pasa.
Universo PROPIO (documentos_url): recursivo bajo la raiz, porque la URL vive
tambien en doc/; fuera AUDITORIA.md y BACKLOG.md (registro) y doc/preprints/
(entrada 28), que se imprimen como exclusion.

ATADO B, LAS FILAS (§841). Las tablas de metricas de ARQUITECTURA.md, PAPER.md y
PAPER_EN.md dan el tamano de las pruebas del pago por su banda: la de envio y la de
cobro, BANDA_ENVIO_B y BANDA_COBRO_B de `mod tests` en metrics.rs; la del pago,
PUBLICADA_PAGO_MIN_B y PUBLICADA_PAGO_MAX_B; y la fila «envio o cobro» de la tabla de
Groth16 de ARQUITECTURA.md, la union de las dos. FILAS_BANDA dice que filas, y cada una
tiene que estar una vez en su documento y llevar en su ultima celda esa banda y ninguna
otra, en bytes con el separador de miles de su idioma. Quien mueva una banda mueve esas
filas, o es ROJO.

RETIRADAS (§841). Cifras que el arbol publico y que se retiraron por rancias o sin
fuente: RETIRADAS. Solo pueden quedar como cita de lo que se decia: en la misma linea o
en la misma frase, antes de la cifra, un «decia», «decian», «it said» o la marca ⚠️.
Universo: el del ATADO F sin sus .rs, mas los comentarios de todos los .rs de crates/.
Fuera, con su razon impresa, EXCLUIDOS_RETIRADAS y PREFIJOS_EXCLUIDOS_RETIRADAS; y las
frases de CITAS_RETIRADAS, cada una con su ancla y su razon: un ancla que ya no casa es
ROJO, para que la excepcion no sobreviva a su frase. Ceguera declarada: no ve una cifra
en el codigo de un .rs (solo sus comentarios de linea entera), ni dentro de un bloque de
codigo de un .md, ni escrita de otra forma («1.12 ms» si, «1,12 milisegundos» no). Y
la marca no tiene que citar la cifra: un «decia» en cualquier sitio de la frase, antes
de ella, cuenta («Esto no lo decia nadie, pero pesa 61.966 B.» pasa).
"""
import difflib
import math
import os
import re
import sys

# ⚠️ §796: hasta aqui la raiz salia siempre de la ruta del fichero. `CANON_RAIZ` la sobreescribe,
# como en canon.sh, check_nucleo y check_dominios: existe para PROBAR el gate en negativo sobre una
# copia, que sin ella resolveria su raiz a si misma.
RAIZ = os.environ.get("CANON_RAIZ") or os.path.dirname(
    os.path.dirname(os.path.abspath(__file__)))
FUENTE = os.path.join(RAIZ, "crates", "zk-ssl", "src", "metrics.rs")
VENTANA = 2

EXCLUIDOS = {
    "AUDITORIA.md": "registro historico: una cifra vieja en un asiento es correcta",
    "BACKLOG.md": "registro: las entradas no se reescriben (la 22 cita la cifra de su fecha)",
    "VISION.md": "nota 98: la seccion 3.8 ENTERA deriva de la era de UN PASO "
                 "-59 MB/mil, 59 GB/dia x3, 131 MB/dia, 463x, 137 B por entrada-. "
                 "No es una cifra rancia: es un analisis. Corte propio",
}

CONTEXTO = re.compile(
    r"mil pagos|mil transferencias|mil operaciones|por cada mil|unidades SI|"
    r"thousand transfers|thousand complete payments|per thousand|SI units",
    re.I,
)
HISTORICA = re.compile(
    r"versiones anteriores|versi[oó]n anterior|version anterior|"
    r"earlier version|previous version|retirada desde entonces",
    re.I,
)
CIFRA = re.compile(r"(?<![\d.,])(\d{2,3})(?:[.,](\d))?\s*(MiB|MB)\b")

# ATADO C (S403)
URL_REPO = "https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers"
TOKEN_VIEJO = "ZK-SSL-ZK-Sovereign-Settlement-Layer"
CFF = os.path.join(RAIZ, "CITATION.cff")
EXCLUIDOS_URL = {
    "AUDITORIA.md": "registro historico: un asiento cita la URL de su fecha",
    "BACKLOG.md": "registro: las entradas no se reescriben",
}
PREFIJOS_EXCLUIDOS_URL = ("doc/preprints/",)

# ATADO D (S549) - el ESTADO de un RFC contra lo que los documentos publican de el.
# El vocabulario lo dicta spec/rfc/PROCESO.md (BORRADOR -> PROPUESTO -> ACEPTADO ->
# FINAL, o RETIRADO); aqui va la cara que cada estado tiene PUBLICADA, por idioma.
DIR_RFC = os.path.join(RAIZ, "spec", "rfc")
PLANTILLA = "0000"
ESTADOS = {
    "ACEPTADO": ("ACCEPTED", ("aceptado", "aceptados", "accepted")),
    "PROPUESTO": ("PROPOSED", ("propuesto", "propuestos", "proposed")),
}
# Los sitios que hoy publican la CUENTA entera. No es un tope: si aparece uno nuevo se
# cruza igual. Esta lista existe para que uno que DEJE de publicarla no pase callando.
CARDINALES = ("README.md", "README_EN.md", "RESUMEN_BILINGUE.md", "RESUMEN_EJECUTIVO.md")
FILA_RFC = "spec/README.md"
UMBRAL_CARDINAL = 3   # tres numeros en una linea; con menos es una mencion, no una cuenta
# El consumidor de un numero se deriva de su PRODUCTOR, que es `rfcs()`: cuatro digitos.
# `000\d` se quedo en el 0009 -- el techo que el S555 midio -- y `\d{4}` leeria un ano o
# una posicion de fichero como RFC. `0\d{3}` cubre la numeracion correlativa entera sin
# alcanzar fuera de ella, y deja viva la regla SOBRA. La puerta D.0 vigila el techo.
NUM_RFC = re.compile(r"\b(0\d{3})\b")
PAL_ESTADO = re.compile(r"\b(aceptad[oa]s?|propuest[oa]s?|accepted|proposed)\b", re.I)
RANGO_RFC = re.compile(r"0\d{3}\s*[-\u2013]\s*0\d{3}")

# ATADO E (S617, la 113) - la CUENTA de ficheros de spec/vectors/ contra lo que los README
# publican de ella. El §616 midio que la tabla «Estado» decia 380 cuando habia 419: el §610 y el
# §614 anadieron vectores sin moverla, y ningun cerrojo miraba esa cifra -check_figures vigila
# las de tests-. La cuenta es la del arbol: todo fichero bajo spec/vectors/, recursivo. La cita
# es «(N ficheros» o «(N files» en una linea que nombra `spec/vectors/`, y los sitios que la
# publican estan en la lista para que uno que DEJE de publicarla no pase callando.
DIR_VECTORES = os.path.join(RAIZ, "spec", "vectors")
SITIOS_VECTORES = ("README.md", "README_EN.md")
CUENTA_VECTORES = re.compile(r"\((\d+)\s+(?:ficheros|files)\b")

# ATADO G (§846, tarea 39 de la lista del autor) - el AUTOR y la ASISTENCIA GenAI en la cabecera
# de cada RFC. GENAI.md dice que cada RFC declara en su cabecera, junto al autor, las sesiones en
# las que se redacto, y hasta aqui ningun cerrojo lo miraba. Universo: spec/rfc/NNNN-*.md, el
# mismo de `rfcs()`; la plantilla 0000, aparte, con su forma de plantilla, y si falta, ROJO.
# Cabecera: las lineas antes del primer `## `. Cada RFC lleva en ella exactamente una «- **Autor:**
# <nombre>», con un nombre de NOMBRES_AUTOR, y exactamente una «- **Asistencia GenAI:** ...», con
# sus lineas de continuacion (las sangradas que la siguen). En esa vineta, ningun identificador de
# modelo: MODELO reconoce la FORMA, un «claude-» seguido de palabras con guion y un digito, en
# mayusculas o en minusculas, y «Claude», una palabra con mayuscula o ninguna, y un digito; no
# nombra ninguna familia. La formula de la casa, «Claude, en una sesion de Claude Code en la
# nube», pasa. Los nombres salen de GENAI.md («Autoria y responsabilidad»): si uno deja de estar
# alli, ROJO, para que la lista no sobreviva a su fuente. Ceguera declarada: no mira el resto del
# RFC, ni un identificador escrito de otra forma (una familia sin digito, «CLAUDE» en mayusculas
# sin guion, un nombre en otra vineta); no exige el enlace a GENAI.md de la vineta, ni que diga
# sesiones. Y no ve un RFC fuera de spec/rfc/ ni con otro nombre de fichero.
NOMBRES_AUTOR = ("Ángel José Toranzo Portela", "Angel Jose Toranzo Portela",
                 "Angel Toranzo Portela")
GENAI = os.path.join(RAIZ, "GENAI.md")
AUTOR_RFC = re.compile(r"^- \*\*Autor(es)?:\*\*\s*(.*)$")
VINETA_GENAI = re.compile(r"^- \*\*Asistencia GenAI:\*\*\s*(.*)$")
MODELO = re.compile(r"(?i:\bclaude-(?:[a-z]+-)*\d)|\bClaude(?:\s+[A-Z][a-z]+)?\s+\d")
# Las excepciones, cada una con la frase del asiento que la explica y el asiento donde esta. La
# frase tiene que seguir en AUDITORIA.md, dentro de su asiento, y la excepcion tiene que seguir
# haciendo falta: si la cabecera cambia, ROJO, y se decide otra vez. El 0002 dice «(mesa)» y no
# lleva vineta de GenAI desde antes de que GENAI.md existiera; el §564, que puso las vinetas en
# las demas, no lo toco, y el autor decidio el 2026-10-15 dejarlo asi, declarado (§846).
AUDITORIA = os.path.join(RAIZ, "AUDITORIA.md")
EXCEPCIONES_G = {
    "0002": ("§564", "El `0002`, que dice `(mesa)`, no se toco.",
             "- **Autores:** (mesa)"),
}
# Lo medido al nacer, en el §846: 21 RFC sin la plantilla. Un suelo, como los del ATADO F.
MINIMO_RFC_G = 21

# ATADO F (§796, tarea 40 del plan) - las CIFRAS DE NIVEL publicadas contra las CONSTANTES de las
# que salen. ⚠️ §796: hasta aqui ningun cerrojo las miraba. El §698 las corrigio a mano en diez
# sitios, el §708 anadio la del termino DEEP, y los dos dejaron escrito lo mismo: «los tests del
# §697 atan las cifras, no la prosa que las cita» (doc/blueprint-v2.md:327, ZK-2). Este atado ata
# la prosa.
#
# DE DONDE SALEN. Medirlas exige compilar Rust; DERIVARLAS, no: las da una funcion cerrada de
# winter-air (`proof/security.rs`) sobre las opciones y la longitud de la traza oculta, y las dos
# cosas son constantes del arbol. Tres eslabones:
#   F.0  las CONSTANTES, leidas del fuente por estructura: `proof_options()` de la capa,
#        `opciones()` de la edad (las mismas, o ROJO), `CICLO`, el TRACE_LENGTH de las familias
#        que prueba el test de la capa, las cuatro opciones del circuito de comparacion, y lo que
#        FIJAN los tests: CONJETURADA_MINIMA, LDR_MINIMA y UDR_ACTUAL en `nivel_de_produccion.rs`;
#        PISO_LDR, CONJETURADA_CON_DEEP, SIN_PISO, CONJETURADA_MINIMA y UDR_ACTUAL en
#        `tests_nivel`, que no tiene LDR_MINIMA: alli el piso va por `m`. UDR_ACTUAL es un VALOR,
#        no un minimo;
#   F.1  la REPLICA de la funcion, aqui, letra a letra y con la aritmetica de f64 de Rust (division
#        por cero, el NaN que `f64::min` descarta, la conversion saturada a u64), CONTRASTADA en
#        tres frentes:
#        a) con lo que los tests asiertan corriendo la funcion real. IGUALDAD solo hay en dos: la
#           UDR, = UDR_ACTUAL (crates/zk-ssl-air/src/lib.rs:1040, nivel_de_produccion.rs:170), y el
#           DEEP en decimas, = CONJETURADA_CON_DEEP (zk-ssl-air/src/lib.rs:1229), que alli da la
#           replica del propio test, `bits_deep`, contrastada con la UDR real de dos variantes
#           (:1206-1215). La LDR, el test solo la exige >= su piso (:1043), y la de las familias,
#           >= LDR_MINIMA (nivel_de_produccion.rs:167): que cada piso sea la LDR exacta lo dice lo
#           MEDIDO en el §697 («Los pisos son los valores de hoy», AUDITORIA.md:48329), no el
#           test. Aqui se exige la igualdad: si la funcion sube sobre un piso, el test pasa y este
#           gate cae, y el piso sube con su asiento. ⚠️ §839: las dos citas al test decian :144 y
#           :141, antes de que ganara la subida de congelados y las longitudes medidas;
#        b) con los KAT del fork, KAT_FORK: los 18 asserts de `mod tests` de security.rs con lote
#           lineal, valores de la funcion real -el canon corre esos tests en el nivel sello, los
#           49 de upstream (tools/canon.sh:123)-, fuera del regimen de produccion: extension
#           cuadratica y cubica, blowup 2 a 16, plegado 2 a 8, traza de 2^8 a 2^22 filas;
#        c) ANCLADA a su fuente, y SOLO en lo que aqui se dice. Las anclas son lo UNICO de este
#           gate que lee el CODIGO del fork, crates/winter-air/src/ -a) compara la replica con
#           constantes de los tests, y b), con KAT_FORK congelado; F.2 lee la prosa de los README
#           del fork-, y de ese codigo vigilan dos cosas y nada mas. CODIGO_FORMULA: las 213
#           lineas de codigo de security.rs HASTA `mod tests` (de :8 a :346; desde el
#           `#[cfg(test)]` de :351 hasta el final, :1391, no se lee), linea a linea y en orden, sin
#           sangria y sin las lineas blancas ni las de comentario `//` -lo de dentro de un `/* */`
#           cuenta como codigo-. Y CABLEADO: estas 21 funciones de crates/winter-air/src/, cada una
#           ENTERA, de la firma a la llave de cierre y sin los atributos `#[...]` que la preceden,
#           200 lineas, con las mismas reglas, y cada una las veces que dice su cabecera en
#           CABLEADO (hoy, una), contadas hasta el `#[cfg(test)] mod tests {` de su fichero
#           (proof/mod.rs no lo tiene: su `mod tests;` de :40 no corta, y se lee entero). Cualquier
#           otra linea del codigo del fork, tambien una por la que pase una entrada de la
#           formula, NO se vigila aqui: la ceguera nombra lo censado -no es una lista cerrada- y
#           las mutaciones medidas en VERDE. Funcion a funcion de la replica, lo que copia y su
#           ancla:
#           - `_conjeturada` y `_bits_consultas`: ConjecturedSecurity::compute (security.rs:30-48),
#             a la que llama Proof::conjectured_security (proof/mod.rs:100-106) con
#             Context::options y Context::num_modulus_bits (proof/context.rs:95-97 y :81-92);
#           - `_ldr` y `_ldr_para_m`: ProvenSecurity::compute (security.rs:77-129),
#             proven_security_protocol_for_given_proximity_parameter (:150-220) y compute_upper_m
#             (:296-306), a las que llama Proof::proven_security (proof/mod.rs:111-131) con
#             Proof::options y Proof::trace_info (:80-82 y :85-87), Context::trace_info y
#             Context::num_constraints (proof/context.rs:63-65 y :100-102), y TraceInfo::length y
#             TraceInfo::width (air/trace_info.rs:154-156 y :147-149);
#           - `_udr`: proven_security_protocol_unique_decoding (security.rs:223-285), a la que llama
#             ProvenSecurity::compute, y `_deep`, su termino epsilon_2 (:255-257) sin el minimo;
#           - `_log2`, `_min_f64`, `_u64` y `_div`: la aritmetica de f64 de esas funciones, con
#             log2, sqrt, powf y ceil (security.rs:308-346), el `fold` con `f64::min` (:219, :276 y
#             :284) y los `as u64`;
#           - lo que esas funciones leen de las opciones: ProofOptions::num_queries,
#             ::blowup_factor, ::grinding_factor, ::field_extension, ::to_fri_options,
#             ::constraint_batching_method y ::deep_poly_batching_method (options.rs:211-213,
#             :221-223, :232-234, :241-243, :254-258, :275-277 y :289-291), y FieldExtension::degree
#             (:353-359), que copia GRADO_EXTENSION;
#           - de donde salen opciones y longitud: ProofOptions::new (options.rs:132-185), cuyos
#             ocho argumentos lee F.0 POR POSICION -el lote lineal, `lotes`, incluido-;
#             Context::new (proof/context.rs:35-57), que guarda opciones, traza y modulo, y
#             TraceInfo::new_multi_segment (air/trace_info.rs:84-139), que guarda la longitud;
#           - las dos longitudes, 2T y 2 * CICLO * 2^m: ANCLAS_LONGITUD, en el fuente que las aplica
#             (crates/zk-ssl/src/lib.rs:255 y crates/zk-ssl-air/src/lib.rs:1027 y :1187), fuera
#             del fork: cada una, una linea cuyo codigo -sin sangria ni comentario de cola `//`-
#             es la regla ENTERA, contada en el fichero entero, que sale las veces dichas (1 y 2).
#           De las FUNCIONES del resto de proof/mod.rs, proof/context.rs, air/trace_info.rs y
#           options.rs, estas no las usan ConjecturedSecurity ni ProvenSecurity, y no se anclan:
#           Proof::lde_domain_size y Context::lde_domain_size, Context::field_modulus_bytes, los
#           to_elements de Context, TraceInfo y ProofOptions, TraceInfo::meta, ::is_multi_segment,
#           ::main_trace_width, ::aux_segment_width, ::num_segments, ::num_aux_segments,
#           ::get_aux_segment_width y ::get_num_aux_segment_rand_elements,
#           ProofOptions::with_partitions, ::domain_offset y ::partition_options,
#           FieldExtension::is_none y PartitionOptions entero. Las de la serializacion
#           -FieldExtension::get_size_hint incluida- y Proof::new_dummy, que si estan en el camino
#           de los tests a la funcion, van en la ceguera con lo demas que la replica usa y NO se
#           ancla. La lista es de FUNCIONES: las constantes y los struct y enum sin ancla van en
#           la ceguera con su nombre; de los `use` y los `mod` solo se nombran `mod security;` y
#           `use security::{...}`, y los demas -`mod context;` (proof/mod.rs:18) entre ellos-
#           tampoco se vigilan. Lo que da ROJO
#           FORMULA, y nada mas: una linea de codigo de security.rs ANTES de `mod tests` que
#           cambia, sobra o falta, con su numero de linea; y una de las 21 funciones del cableado
#           que ya no sale tal cual sus veces, con la linea donde se aparta del ancla -o la de la
#           vez de mas, si sale de mas-; en `:0` si falta la primera linea, si no hay una parecida
#           o pasadas ocho diferencias. Un cambio del fork fuera de esas lineas solo da rojo si
#           cambia lo que esas dos anclas leen -una copia de mas de un bloque anclado, o un
#           `#[cfg(test)]` y un `mod tests {` que cortan antes-; si no, VERDE, aunque cambie la
#           formula que se compila (en la ceguera, los medidos);
#   F.2  la PROSA contra la replica: cada cifra de nivel de un documento vivo casa con la que se
#        deriva, y las tres tablas -las dos de SECURITY.md §3.11 y la de FIVE_BACKENDS.md §4- se
#        atan celda a celda.
#
# ACOPLAMIENTO con la zona del fork. F.1 c) ancla, LITERALES, las 213 lineas de codigo de
# security.rs hasta `mod tests` y las 200 de las 21 funciones del cableado, en cinco ficheros de
# crates/winter-air/src/ (como ya hace con tres lineas el test del §708,
# crates/zk-ssl-air/src/lib.rs:1153-1169). Un rustfmt o una migracion de winterfell que les cambie
# la FORMA pone este gate en ROJO FORMULA aunque la formula no cambie. Que hacer: el sello que toque
# esos ficheros actualiza CODIGO_FORMULA y CABLEADO en el MISMO sello. Que la replica siga siendo la
# funcion NUEVA no lo dicen a) ni b), que no leen el fork y saldrian verdes igual: lo dicen los
# tests de Rust que el canon corre en el nivel sello, los del §697 y el §708 en zk-ssl-air y los
# 49 de upstream en winter-air (tools/canon.sh:122-123). Si cambian los asserts de `mod tests` de
# security.rs, se copian otra vez a KAT_FORK; si cambia la formula, cambia la replica, y las cifras
# publicadas con su asiento.
#
# Reglas de F.2, declaradas ANTES de mirar:
#   - la unidad es la FRASE: un parrafo unido y cortado en punto o punto y coma ante mayuscula;
#     una celda en las tablas. Las cifras de una frase son UNA afirmacion y tienen que salir TODAS
#     de UNA configuracion: la de produccion, o una de las cuatro filas de la comparacion;
#   - la frase elige configuracion por sus MARCAS: «produccion», «ocultas», «42 consultas»... la
#     atan a la de produccion; «comparacion», «32 consultas», «sin molienda»... a las de la
#     comparacion. Sin marca hereda la de la frase anterior hasta el siguiente encabezado, porque
#     «Ahi, los 127 bits...» habla de lo que dijo la frase de antes. Sin marca y sin herencia, o
#     con las dos marcas, vale cualquiera, pero una sola para todas sus cifras;
#   - una cifra de nivel SOLA en su frase -una conjeturada, una UDR, una LDR o un par «UDR/LDR»
#     demostrables, sin otra cifra que elija fila- no elige configuracion: sin la marca de la
#     comparacion en su PROPIA frase, solo vale la de produccion, haya herencia o no. Sin esta
#     regla «~127 bits conjeturados» (doc/ecst/ECST.md:718) pasaba con la conjeturada de otra fila:
#     63, 95 o 128, y «26/47 bits demostrables» en su lugar, con el par de la segunda;
#   - lo que va entre «» es una CITA de otro texto, no lo que se publica: vale cualquier
#     configuracion, y sus marcas no cuentan para la frase;
#   - los rangos se atan EXACTOS: «80-88» son la LDR de la familia mas larga y la de la mas corta
#     de las que prueba el test de la capa, y «112,8-116,8», su DEEP. Una LDR suelta, sin su `m`
#     ni su longitud, solo tiene que estar entre las que da produccion: «82 en el envio» pasaria;
#   - «≥» cita al TEST, no a la funcion: «LDR ≥ 80» casa con LDR_MINIMA, «conjeturada ≥ 127» con
#     CONJETURADA_MINIMA; «UDR = 59», con la UDR que se deriva Y con UDR_ACTUAL;
#   - una cifra de otro sistema se SALTA con su ancla de texto y se imprime, y un ancla que ya no
#     encuentra nada es ROJO: una exclusion muerta tapa la proxima;
#   - el rojo se dice contra la configuracion que menos falla, y en empate, contra la comparacion
#     si la frase no es de produccion; pero una frase con una cifra que solo da produccion -un
#     rango, una `m`, una longitud, un «≥», los bits que «dan» las consultas: SOLO_PRODUCCION- se
#     dice contra produccion: «los 95 bits declarados son conjeturados (112,8-116,8 con el termino
#     DEEP...)» es un 95 que sobra, no un DEEP que le falte a la fila de la extension cuadratica;
#   - prueba de vida: los sitios de SITIOS_NIVEL publican cada uno al menos una cifra de
#     PRODUCCION, y el censo no baja de MINIMO_CIFRAS_F ni las celdas de MINIMO_CELDAS_F, lo
#     medido en el §796.
#
# El universo es el del ATADO C sin CITATION.cff, mas los comentarios de FUENTES_RS_NIVEL, y hereda
# las exclusiones del C -AUDITORIA.md, BACKLOG.md y doc/preprints/-, que se imprimen con la razon
# de este atado (HEREDADOS_NIVEL); si las del C cambian y estas no, es ROJO.
#
# Ceguera declarada:
#   - una cifra demostrable sin UDR ni LDR al lado no se mira: «128 bits demostrables» es casi
#     siempre un objetivo, no un nivel; ni «techo de 63 bits», ni el «128 que pone el hash»;
#   - una cifra de nivel sola en una frase CON la marca de la comparacion vale la de cualquiera de
#     sus cuatro filas, o tambien la de produccion si la frase lleva las dos marcas: «en la
#     comparacion, 95 bits conjeturados» pasaria con 63. Y entre «», la de cualquier configuracion.
#     De las primeras hoy no hay ninguna -la cuenta se imprime-: las cifras de la comparacion van
#     con otra que elige su fila;
#   - una frase de VARIAS cifras, todas de nivel -conjeturada, UDR, LDR, par- y sin marca propia,
#     toma la configuracion heredada, o cualquiera sin herencia, con tal de que sea una para todas:
#     «63 conjectured vs 26-24 provable» en la celda de doc/ZENODO.md:125 pasaria con la primera
#     fila. La regla de la cifra sola no se puede extender a ellas sin romper lo que hay: las dos
#     que hoy cuentan son de la comparacion de verdad -FIVE_BACKENDS.md:151, «Ahi, los 127
#     bits...», por herencia, y la celda de ZENODO.md:125, «127 conjectured vs 29-63», con su
#     marca en la celda de al lado-. La cuenta se imprime;
#   - las cifras del circuito de comparacion salen SOLO de la replica: ningun test las fija
#     (`real_proof_end_to_end_with_metrics` imprime y no compara), y ninguno de los 18 KAT_FORK
#     tiene el regimen de sus tres primeras filas -sin molienda, y la primera sin extension: todos
#     llevan 16 o 20-. Casan con lo remedido en el §698 con la funcion real
#     (AUDITORIA.md:48582-48587);
#   - KAT_FORK esta CONGELADO: si el fork cambia sus tests, esta tabla no lo ve; si cambia la
#     formula en las lineas que anclan c), lo ven ellas -fuera de ellas, no: abajo-, y que la
#     replica nueva siga siendo la funcion lo dicen los tests de Rust del canon, no esta tabla (ver
#     ACOPLAMIENTO). La replica solo sabe el lote lineal: los 11 asserts con lote algebraico de
#     esos tests quedan fuera, y produccion exige el lineal (F.0);
#   - lo censado que la replica usa y NO esta anclado en c), por su nombre -lo censado, no una
#     lista cerrada: lo que falte en ella tampoco se vigila-:
#     . de winter-fri, winter-math y winter-crypto, que vienen del registro y no viven en el arbol:
#       FriOptions::folding_factor y FriOptions::num_fri_layers, que llama security.rs:272-273
#       -`_udr` toma el plegado de las opciones y copia el bucle de la segunda-; FriOptions::new,
#       que llama `to_fri_options`; el modulo de Goldilocks que guarda Context::new
#       (`B::get_modulus_le_bytes()`, proof/context.rs:53), BITS_CAMPO_BASE = 64, y el
#       COLLISION_RESISTANCE de Blake3_256, RESISTENCIA_HASH = 128. Y la aritmetica de f64 de Rust
#       y de libm, que la replica imita con la de Python. Los cubre el contraste de F.1, no un
#       ancla. Medido, cambiando la herramienta en una copia: con RESISTENCIA_HASH = 127 o 256 caen
#       los KAT de security.rs:713, :738 y :799, y la fila cubica de FIVE_BACKENDS.md:148, pero
#       ningun REPLICA; con BITS_CAMPO_BASE = 63 o 65, los REPLICA de los pisos y siete u ocho KAT;
#       con el plegado, y no el plegado menos uno, en el termino de FRI, los KAT de :478, :500 y
#       :1215. De num_fri_layers solo cuenta si es cero: el termino es el minimo de un valor que no
#       depende de la capa (security.rs:274-276), asi que con otro numero positivo la funcion y la
#       replica dan lo mismo -con cinco capas de mas en `_udr`, VERDE-, y sin capas cae solo el KAT
#       de :500;
#     . el camino del probador hasta el contexto: donde fija la longitud oculta y las opciones
#       (crates/winter-prover/src/lib.rs:600-629, `2 * t` en :620 y `self.options()` en :628), el
#       AIR que las reparte (air/oculta.rs:57-91), el AirContext que las guarda
#       (air/context.rs:90-161) y los accesores de `Air` que las devuelven (air/mod.rs:397-399 y
#       :403-405), la llamada a Context::new del canal (crates/winter-prover/src/channel.rs:58-64,
#       desde lib.rs:641-645), y TraceInfo::new y ::with_meta (air/trace_info.rs:53-69), que hacen
#       la traza de dentro. Las opciones de la prueba real las mira, en Rust, el test de la capa
#       (crates/zk-ssl/src/nivel_de_produccion.rs:68), no este gate; y 2T, la replica lo toma de lo
#       que la capa EXIGE (ANCLAS_LONGITUD), y ese test se lo exige a sus pruebas reales (:69): una
#       prueba de otra longitud la rechaza la capa, y este gate no lo ve;
#     . la ida y vuelta por bytes: Proof::to_bytes, ::from_bytes, ::write_into y ::read_from
#       (proof/mod.rs:137-158 y :204-240), y los write_into y read_from de Context
#       (proof/context.rs:142-185), TraceInfo (air/trace_info.rs:240-333), ProofOptions
#       (options.rs:307-341), FieldExtension (:362-386, con su get_size_hint, :369-371) y
#       BatchingMethod (:485-507). La replica da el nivel del contexto tal como se CONSTRUYE; que
#       leido de bytes sea el mismo lo mira, en Rust, el test de la capa -las opciones
#       (nivel_de_produccion.rs:68) y la forma (:69), no el modulo-, no este gate; ⚠️ §839: aqui
#       y arriba decia :58 y :59, antes de que el modulo del test creciera en su cabecera
#     . las definiciones de FieldExtension y BatchingMethod (options.rs:47-54 y :479-483), cuyos
#       nombres leen F.0 y security.rs, y las de los struct ProofOptions (:95-105), Context
#       (proof/context.rs:17-22), TraceInfo (air/trace_info.rs:24-30) y Proof (proof/mod.rs:56-76),
#       cuyo `context` reemplaza el test de la edad: un tipo cambiado sin tocar las funciones
#       ancladas no compila; las constantes que comparan los asserts de esas funciones
#       (options.rs:18-28 y air/trace_info.rs:34-40): un assert que salta para la prueba, no cambia
#       una cifra; y ProofOptions::MIN_BLOWUP_FACTOR (options.rs:118), la de :20 vuelta a publicar,
#       que lee air/transition/degree.rs:114 y ningun assert de esas funciones;
#     . de comprobar_forma, fuera de la linea anclada, la forma que lee
#       (crates/zk-ssl/src/lib.rs:248-253) y la comparacion (:256); y del test de la edad, lo que
#       rodea la linea anclada las dos veces que sale: la traza y el contexto que construye
#       (crates/zk-ssl-air/src/lib.rs:1028-1032, y tras :1187, el cierre `contexto` de :1188-1195)
#       y Proof::new_dummy (proof/mod.rs:161-198), cuyo contexto ese test reemplaza.
#     Medido, sobre una copia del arbol y en VERDE como se declara: `4 * t` en
#     winter-prover/src/lib.rs:620, otra traza en channel.rs:61, `l / 4` en air/oculta.rs:59, el
#     modulo sin su primer byte en el read_from de Context (proof/context.rs:180), la longitud por
#     dos en el de TraceInfo (air/trace_info.rs:315), MAX_NUM_QUERIES = 40 (options.rs:18),
#     `length / 2` en TraceInfo::new (air/trace_info.rs:54) e `info.length() / 2` en
#     crates/zk-ssl/src/lib.rs:252;
#   - lo que las anclas de c) NO leen, aunque por ahi se compile otra formula: CODIGO_FORMULA
#     compara las lineas de security.rs HASTA `mod tests`, el CABLEADO solo cuenta cuantas veces
#     sale cada bloque entero, y `_codigo` solo quita las lineas `//`. Fuera de security.rs,
#     nada lee los atributos `#[...]` de una de las 21 funciones del cableado ni los del `impl` o el
#     `mod` que la contiene, ni otra `fn` del mismo nombre, ni distingue lo de dentro de un `/* */`
#     (en security.rs, antes de `mod tests`, un atributo o una `fn` de mas son lineas que
#     CODIGO_FORMULA compara, y dan rojo). Medido, sobre una copia del arbol con
#     CANON_RAIZ, una mutacion cada vez, y las cinco en VERDE (rc=0):
#     . tras `mod tests`: `const SOBRA: u32 = 1;` anadida tras la llave que cierra el modulo de
#       tests de security.rs (:1391). Una definicion con el nombre de una de :8-346 no compila,
#       pero lo que va ahi no se lee;
#     . un atributo: `#[cfg(any())]` en lugar del `#[allow(clippy::too_many_arguments)]` de
#       options.rs:131, con otra `new` que guarda `(num_queries / 2) as u8` metida tras :185;
#     . una segunda `fn` del mismo nombre: `#[cfg(any())]` ante `num_queries` (options.rs:211),
#       con otra que devuelve `self.num_queries as usize / 2` detras;
#     . un comentario de bloque: `/*` y `*/` alrededor de options.rs:211-213, con esa misma otra
#       `num_queries` detras. En las tres de options.rs la anclada sigue saliendo una vez y se
#       compila la otra;
#     . lo que no esta en ningun bloque: `mod security;` (proof/mod.rs:30) y `use security::{...}`
#       (:13). Con `#[path = "otra.rs"] mod security;` en :30 y un proof/otra.rs igual que
#       security.rs salvo `as u32 / 2` en su :40, se compila otra formula y CODIGO_FORMULA sigue
#       leyendo security.rs;
#   - ANCLAS_LONGITUD cuenta, en el fichero ENTERO, las lineas cuyo codigo -sin sangria ni
#     comentario de cola `//`- es la regla: no sabe de `mod tests` ni de `/* */`, y una linea
#     dentro de un `/* */` de varias lineas le cuenta como codigo -medido en VERDE: la de
#     crates/zk-ssl-air/src/lib.rs:1027 entre una linea `/*` y otra `*/`, con
#     `let filas = 4 * (CICLO << m);` detras-, ni otra linea que redefina `exige` o `filas`
#     detras de la anclada, ni un `#[cfg]` sobre ella -medidas en VERDE: `let exige = (ancho + 1,
#     ancho_aux, aleatorios_aux, 4 * longitud);` tras crates/zk-ssl/src/lib.rs:255, sola y con
#     `#[cfg(any())]` delante de :255-; y su rojo, LONGITUD, sale en `:0`, sin la linea;
#   - que circuitos lleva cada fila de la tabla de longitud fija es prosa: se atan sus filas, su T,
#     su LDR y su DEEP, no los nombres de la columna «pruebas».
#
# Falsadores (§796), sobre una copia del arbol con CANON_RAIZ, una mutacion cada vez, y todos en
# ROJO: una cifra de la prosa en FIVE_BACKENDS, PRINCIPIOS, PAPER, PAPER_EN y el comentario de
# lib.rs; una celda de cada una de las tres tablas; «≥ 80» y una cita entre «»; la constante, 42 ->
# 34 consultas; un piso de PISO_LDR; el TRACE_LENGTH de la familia mas larga; un sitio que deja de
# publicar, y el ancla de RISC Zero. En el fork, cada una con su ROJO FORMULA y su linea -en el
# cableado, la linea donde la funcion se aparta del ancla-: security.rs:93 y :125 -> `as u64 + 1`,
# :272 -> `as f64 + 1.0`, :159 y :231 -> `as f64 + 1.0`, `let m = m as f64 + 1.0;` (:160),
# `max_by_key` -> `min_by_key` (:102) y `let h = h as f64 * 2.0;` (:297); una linea nueva tras
# :276, la de :300 borrada, y :32 y :33 -los dos parametros u32- trocadas; y en el cableado, los
# dos argumentos u32 de la llamada de proof/mod.rs:103-104 trocados (:103), la longitud por dos
# en :126, Proof::options con otras opciones (:81), `num_bits -= 7` (proof/context.rs:88),
# num_constraints por dos (:101), el modulo sin su primer byte en Context::new (:53) y una linea
# metida en ella (:50), `self.trace_length / 2` (air/trace_info.rs:155), width mas uno (:148),
# `let trace_length = trace_length / 2;` metida en new_multi_segment (:107) y el doble de la
# longitud en lo que guarda (:136), dos argumentos de `ProofOptions::new` trocados
# (options.rs:133), `num_queries as u8 / 2` (:175), otras particiones (:181), `batching_deep:
# BatchingMethod::Algebraic,` (:183), `let num_queries = num_queries / 2;` metida entre sus
# asserts y su `Self {` (en la linea metida), `pub fn new(` por `pub const fn new(` (:132, con la
# linea mas parecida), num_queries() mas uno (:212), el plegado y el resto trocados en
# `to_fri_options` (:257), `self.batching_constraints` y `self.batching_deep` ->
# `BatchingMethod::Algebraic` (:276 y :290) y `Self::Quadratic => 3` (:356). Un assert de
# `mod tests` (:391) o un comentario de linea entera no lo tocan, y un comentario de cola en una
# linea de codigo, si. En ANCLAS_LONGITUD, ROJO LONGITUD en `:0` -contadas sobre el texto crudo,
# con sus comentarios, salian VERDE-: la regla cambiada con la vieja en un comentario de cola,
# `4 * longitud); // let exige = (... 2 * longitud);` en crates/zk-ssl/src/lib.rs:255 y
# `4 * (CICLO << m); // era: let filas = 2 * (CICLO << m);` en crates/zk-ssl-air/src/lib.rs:1027, y
# la de :1187 cambiada con la vieja en un comentario `//` de linea entera encima; un comentario de
# cola en la regla sin cambiarla, VERDE. Con la regla de la cifra sola, las ocho conjeturadas
# sueltas que sin ella salian en VERDE: doc/ecst/ECST.md:718 y :954 -> 95,
# doc/ecst/ECST_EN.md:725 y :965 -> 63, ARQUITECTURA.md:1423 -> 128, FIVE_BACKENDS.md:277 -> 95,
# doc/integracion-vertical-evaluacion.md:77 -> 128, y doc/INSTITUCIONAL.md:70 -> 95 sin su
# parentesis del DEEP, que solo caia por el suelo de VIDA; con ella extendida al par, «26/47 bits
# demostrables» en lugar de la conjeturada de doc/ecst/ECST.md:718. Con el parentesis del DEEP, el
# 95 de doc/INSTITUCIONAL.md:70 ya se dice contra produccion, «da 127», y no contra la fila de la
# extension cuadratica. Y un valor de KAT_FORK, una exclusion del C sin su razon aqui, y
# SECURITY.md o FIVE_BACKENDS.md borrados: ILEGIBLE, sin traceback. La ceguera, medida: «En la
# comparacion, 63 bits conjeturados.» anadida a FIVE_BACKENDS.md sale VERDE, como se declara, y «63
# conjectured vs 26-24 provable» en doc/ZENODO.md:125, tambien. Fuera de produccion la replica da,
# con q = 33, LDR 79 en el envio, y con q = 34, LDR 80 y UDR 52: lo mismo que el debate del plan
# dio SIN correr la funcion real -ESTIMADO en doc/blueprint-v2.md:308 y :557; la «LDR 79» con 33 de
# :338 no cita medida-, que alli solo corrio con q = 42 (:245, :306-307 y :1115). Eso no la
# contrasta: la contrasta b).
DIR_CRATES = os.path.join(RAIZ, "crates")
FUENTE_OPCIONES = os.path.join(DIR_CRATES, "zk-ssl", "src", "lib.rs")
FUENTE_TEST_CAPA = os.path.join(DIR_CRATES, "zk-ssl", "src", "nivel_de_produccion.rs")
FUENTE_EDAD = os.path.join(DIR_CRATES, "zk-ssl-air", "src", "lib.rs")
DIR_FORK = os.path.join(DIR_CRATES, "winter-air", "src")
FUENTE_FORMULA = os.path.join(DIR_FORK, "proof", "security.rs")
DIR_CIRCUITOS = os.path.join(DIR_CRATES, "stark-experiment", "src")
FUENTE_COMPARACION = os.path.join(DIR_CIRCUITOS, "compliance_real_proof.rs")
CIRCUITO_COMPARACION = "compliance_circuit"
BITS_CAMPO_BASE = 64      # MODULUS_BITS de Goldilocks (winter-math, fuera del arbol)
RESISTENCIA_HASH = 128    # COLLISION_RESISTANCE de Blake3_256 (winter-crypto, fuera del arbol)
GRADO_EXTENSION = {"None": 1, "Quadratic": 2, "Cubic": 3}   # `degree()`, anclada en options.rs
# F.1 c), la primera ancla, LITERAL: CODIGO_FORMULA es el codigo de security.rs HASTA `mod tests`
# (lo de detras no se lee), cada linea sin su sangria y en orden; fuera solo las lineas blancas y
# las de comentario `//`. Es copia del fork de winterfell 0.13.1 (MIT, Copyright (c) Facebook,
# Inc. and its affiliates: crates/winter-air/LICENSE). Si un sello del fork le cambia una linea,
# la actualiza el mismo sello (ver ACOPLAMIENTO, arriba).
CODIGO_FORMULA = tuple(l for l in r"""
use core::cmp;
use crate::{BatchingMethod, ProofOptions};
const GRINDING_CONTRIBUTION_FLOOR: u32 = 80;
const MAX_PROXIMITY_PARAMETER: u64 = 1000;
pub struct ConjecturedSecurity(u32);
impl ConjecturedSecurity {
pub fn compute(
options: &ProofOptions,
base_field_bits: u32,
collision_resistance: u32,
) -> Self {
let field_security = base_field_bits * options.field_extension().degree();
let security_per_query = options.blowup_factor().ilog2();
let mut query_security = security_per_query * options.num_queries() as u32;
if query_security >= GRINDING_CONTRIBUTION_FLOOR {
query_security += options.grinding_factor();
}
Self(cmp::min(cmp::min(field_security, query_security) - 1, collision_resistance))
}
pub fn bits(&self) -> u32 {
self.0
}
pub fn is_at_least(&self, bits: u32) -> bool {
self.0 >= bits
}
}
pub struct ProvenSecurity {
unique_decoding: u32,
list_decoding: u32,
}
impl ProvenSecurity {
pub fn compute(
options: &ProofOptions,
base_field_bits: u32,
trace_domain_size: usize,
collision_resistance: u32,
num_constraints: usize,
num_committed_polys: usize,
) -> Self {
let unique_decoding = cmp::min(
proven_security_protocol_unique_decoding(
options,
base_field_bits,
trace_domain_size,
num_constraints,
num_committed_polys,
),
collision_resistance as u64,
) as u32;
let m_min: usize = 3;
let m_max = compute_upper_m(trace_domain_size);
let m_optimal = (m_min as u32..m_max as u32)
.max_by_key(|&a| {
proven_security_protocol_for_given_proximity_parameter(
options,
base_field_bits,
trace_domain_size,
a as usize,
num_constraints,
num_committed_polys,
)
})
.expect(
"Should not fail since m_max is larger than m_min for all trace sizes of length greater than 4",
);
let list_decoding = cmp::min(
proven_security_protocol_for_given_proximity_parameter(
options,
base_field_bits,
trace_domain_size,
m_optimal as usize,
num_constraints,
num_committed_polys,
),
collision_resistance as u64,
) as u32;
Self { unique_decoding, list_decoding }
}
pub fn ldr_bits(&self) -> u32 {
self.list_decoding
}
pub fn udr_bits(&self) -> u32 {
self.unique_decoding
}
pub fn is_at_least(&self, bits: u32) -> bool {
self.list_decoding >= bits || self.unique_decoding >= bits
}
}
fn proven_security_protocol_for_given_proximity_parameter(
options: &ProofOptions,
base_field_bits: u32,
trace_domain_size: usize,
m: usize,
num_constraints: usize,
num_committed_polys: usize,
) -> u64 {
let extension_field_bits = (base_field_bits * options.field_extension().degree()) as f64;
let num_fri_queries = options.num_queries() as f64;
let m = m as f64;
let rho = 1.0 / options.blowup_factor() as f64;
let alpha = (1.0 + 0.5 / m) * sqrt(rho);
let max_deg = options.blowup_factor() as f64 + 1.0;
let lde_domain_size = (trace_domain_size * options.blowup_factor()) as f64;
let trace_domain_size = trace_domain_size as f64;
let num_openings = 2.0;
let mut epsilons_bits_neg = vec![];
let l = m / (rho - (2.0 * m / lde_domain_size));
let batching_factor = match options.constraint_batching_method() {
BatchingMethod::Linear => 1.0,
BatchingMethod::Algebraic | BatchingMethod::Horner => num_constraints as f64 - 1.0,
};
let epsilon_1_bits_neg = -log2(l) - log2(batching_factor) + extension_field_bits;
epsilons_bits_neg.push(epsilon_1_bits_neg);
let epsilon_2_bits_neg = -log2(
l * l * (max_deg * (trace_domain_size + num_openings - 1.0) + (trace_domain_size - 1.0)),
) + extension_field_bits;
epsilons_bits_neg.push(epsilon_2_bits_neg);
let batching_factor = match options.deep_poly_batching_method() {
BatchingMethod::Linear => 1.0,
BatchingMethod::Algebraic | BatchingMethod::Horner => num_committed_polys as f64 - 1.0,
};
let epsilon_3_bits_neg = extension_field_bits
- log2(
(powf(m + 0.5, 7.0) / (3.0 * powf(rho, 1.5)))
* powf(lde_domain_size, 2.0)
* batching_factor,
);
epsilons_bits_neg.push(epsilon_3_bits_neg);
let epsilon_k_bits_neg = options.grinding_factor() as f64 - log2(powf(alpha, num_fri_queries));
epsilons_bits_neg.push(epsilon_k_bits_neg);
epsilons_bits_neg.into_iter().fold(f64::INFINITY, |a, b| a.min(b)) as u64
}
fn proven_security_protocol_unique_decoding(
options: &ProofOptions,
base_field_bits: u32,
trace_domain_size: usize,
num_constraints: usize,
num_committed_polys: usize,
) -> u64 {
let extension_field_bits = (base_field_bits * options.field_extension().degree()) as f64;
let num_fri_queries = options.num_queries() as f64;
let lde_domain_size = (trace_domain_size * options.blowup_factor()) as f64;
let trace_domain_size = trace_domain_size as f64;
let num_openings = 2.0;
let rho_plus = (trace_domain_size + num_openings) / lde_domain_size;
let alpha = (1.0 + rho_plus) * 0.5;
let max_deg = options.blowup_factor() as f64 + 1.0;
let mut epsilons_bits_neg = vec![];
let batching_factor = match options.constraint_batching_method() {
BatchingMethod::Linear => 1.0,
BatchingMethod::Algebraic | BatchingMethod::Horner => num_constraints as f64 - 1.0,
};
let epsilon_1_bits_neg = -log2(batching_factor) + extension_field_bits;
epsilons_bits_neg.push(epsilon_1_bits_neg);
let epsilon_2_bits_neg =
-log2(max_deg * (trace_domain_size + num_openings - 1.0) + (trace_domain_size - 1.0))
+ extension_field_bits;
epsilons_bits_neg.push(epsilon_2_bits_neg);
let batching_factor = match options.deep_poly_batching_method() {
BatchingMethod::Linear => 1.0,
BatchingMethod::Algebraic | BatchingMethod::Horner => num_committed_polys as f64 - 1.0,
};
let epsilon_3_bits_neg = extension_field_bits - log2(lde_domain_size * batching_factor);
epsilons_bits_neg.push(epsilon_3_bits_neg);
let folding_factor = options.to_fri_options().folding_factor() as f64;
let num_fri_layers = options.to_fri_options().num_fri_layers(lde_domain_size as usize);
let epsilon_i_min_bits_neg = (0..num_fri_layers)
.map(|_| extension_field_bits - log2((folding_factor - 1.0) * (lde_domain_size + 1.0)))
.fold(f64::INFINITY, |a, b| a.min(b));
epsilons_bits_neg.push(epsilon_i_min_bits_neg);
let epsilon_k_bits_neg = options.grinding_factor() as f64 - log2(powf(alpha, num_fri_queries));
epsilons_bits_neg.push(epsilon_k_bits_neg);
epsilons_bits_neg.into_iter().fold(f64::INFINITY, |a, b| a.min(b)) as u64
}
fn compute_upper_m(h: usize) -> f64 {
let h = h as f64;
let ratio = (h + 2.0) / h;
let m_max = ceil(1.0 / (2.0 * (sqrt(ratio) - 1.0)));
assert!(m_max >= h / 2.0, "the bound in the theorem should be tighter");
cmp::min(m_max as u64, MAX_PROXIMITY_PARAMETER) as f64
}
#[cfg(feature = "std")]
pub fn log2(value: f64) -> f64 {
value.log2()
}
#[cfg(not(feature = "std"))]
pub fn log2(value: f64) -> f64 {
libm::log2(value)
}
#[cfg(feature = "std")]
pub fn sqrt(value: f64) -> f64 {
value.sqrt()
}
#[cfg(not(feature = "std"))]
pub fn sqrt(value: f64) -> f64 {
libm::sqrt(value)
}
#[cfg(feature = "std")]
pub fn powf(value: f64, exp: f64) -> f64 {
value.powf(exp)
}
#[cfg(not(feature = "std"))]
pub fn powf(value: f64, exp: f64) -> f64 {
libm::pow(value, exp)
}
#[cfg(feature = "std")]
pub fn ceil(value: f64) -> f64 {
value.ceil()
}
#[cfg(not(feature = "std"))]
pub fn ceil(value: f64) -> f64 {
libm::ceil(value)
}
""".split("\n") if l)
# Y el CABLEADO: estas 21 funciones del fork, por su nombre, cada una ENTERA y en orden -de su
# firma a su llave de cierre, sin sangria y sin las lineas blancas ni las de comentario `//`; sus
# atributos `#[...]` no entran-, con las veces que hoy sale hasta el `#[cfg(test)] mod tests {` de
# su fichero. Cualquier otra funcion del fork fuera de security.rs, tambien una por la que pasen
# las entradas de la formula, no esta aqui y NO se vigila; la ceguera de la cabecera nombra las
# censadas, sin cerrar la lista. Entera, no por trozos: una linea metida entre dos trozos
# anclados (`let num_queries = num_queries / 2;` entre la firma de `ProofOptions::new` y su
# `Self {`) no la ve ningun trozo. La cabecera `@` dice fichero (bajo crates/winter-air/src/),
# veces y funcion; que funcion de la replica pasa por cada una, en F.1 c).
CABLEADO = r"""
@ proof/mod.rs 1 Proof::options
pub fn options(&self) -> &ProofOptions {
self.context.options()
}
@ proof/mod.rs 1 Proof::trace_info
pub fn trace_info(&self) -> &TraceInfo {
self.context.trace_info()
}
@ proof/mod.rs 1 Proof::conjectured_security
pub fn conjectured_security<H: Hasher>(&self) -> ConjecturedSecurity {
ConjecturedSecurity::compute(
self.context.options(),
self.context.num_modulus_bits(),
H::COLLISION_RESISTANCE,
)
}
@ proof/mod.rs 1 Proof::proven_security
pub fn proven_security<H: Hasher>(&self) -> ProvenSecurity {
let num_constraints = self.context.num_constraints();
let num_trace_polys = self.context.trace_info().width();
let num_constraint_composition_polys = self.options().blowup_factor();
let num_committed_polys = num_trace_polys + num_constraint_composition_polys;
ProvenSecurity::compute(
self.context.options(),
self.context.num_modulus_bits(),
self.trace_info().length(),
H::COLLISION_RESISTANCE,
num_constraints,
num_committed_polys,
)
}
@ proof/context.rs 1 Context::new
pub fn new<B: StarkField>(
trace_info: TraceInfo,
options: ProofOptions,
num_constraints: usize,
) -> Self {
let trace_length = trace_info.length();
assert!(trace_length <= u32::MAX as usize, "trace length too big");
let lde_domain_size = trace_length * options.blowup_factor();
assert!(lde_domain_size <= u32::MAX as usize, "LDE domain size too big");
assert!(num_constraints > 0, "number of constraints should be greater than zero");
assert!(num_constraints <= u32::MAX as usize, "number of constraints is too big");
Context {
trace_info,
field_modulus_bytes: B::get_modulus_le_bytes(),
options,
num_constraints,
}
}
@ proof/context.rs 1 Context::trace_info
pub fn trace_info(&self) -> &TraceInfo {
&self.trace_info
}
@ proof/context.rs 1 Context::num_modulus_bits
pub fn num_modulus_bits(&self) -> u32 {
let mut num_bits = self.field_modulus_bytes.len() as u32 * 8;
for &byte in self.field_modulus_bytes.iter().rev() {
if byte != 0 {
num_bits -= byte.leading_zeros();
return num_bits;
}
num_bits -= 8;
}
0
}
@ proof/context.rs 1 Context::options
pub fn options(&self) -> &ProofOptions {
&self.options
}
@ proof/context.rs 1 Context::num_constraints
pub fn num_constraints(&self) -> usize {
self.num_constraints
}
@ air/trace_info.rs 1 TraceInfo::new_multi_segment
pub fn new_multi_segment(
main_segment_width: usize,
aux_segment_width: usize,
num_aux_segment_rands: usize,
trace_length: usize,
trace_meta: Vec<u8>,
) -> Self {
assert!(
trace_length >= Self::MIN_TRACE_LENGTH,
"trace length must be at least {}, but was {}",
Self::MIN_TRACE_LENGTH,
trace_length
);
assert!(
trace_length.is_power_of_two(),
"trace length must be a power of two, but was {trace_length}"
);
assert!(
trace_meta.len() <= Self::MAX_META_LENGTH,
"number of metadata bytes cannot be greater than {}, but was {}",
Self::MAX_META_LENGTH,
trace_meta.len()
);
assert!(main_segment_width > 0, "main trace segment must consist of at least one column");
let full_width = main_segment_width + aux_segment_width;
assert!(
full_width <= TraceInfo::MAX_TRACE_WIDTH,
"total number of columns in the trace cannot be greater than {}, but was {}",
TraceInfo::MAX_TRACE_WIDTH,
full_width
);
if aux_segment_width == 0 {
assert!(
num_aux_segment_rands == 0,
"number of random elements for an empty auxiliary trace segment must be zero"
);
}
assert!(
num_aux_segment_rands <= TraceInfo::MAX_RAND_SEGMENT_ELEMENTS,
"number of random elements required by a segment cannot exceed {}, but was {}",
TraceInfo::MAX_RAND_SEGMENT_ELEMENTS,
num_aux_segment_rands
);
TraceInfo {
main_segment_width,
aux_segment_width,
num_aux_segment_rands,
trace_length,
trace_meta,
}
}
@ air/trace_info.rs 1 TraceInfo::width
pub fn width(&self) -> usize {
self.main_segment_width + self.aux_segment_width
}
@ air/trace_info.rs 1 TraceInfo::length
pub fn length(&self) -> usize {
self.trace_length
}
@ options.rs 1 ProofOptions::new
pub const fn new(
num_queries: usize,
blowup_factor: usize,
grinding_factor: u32,
field_extension: FieldExtension,
fri_folding_factor: usize,
fri_remainder_max_degree: usize,
batching_constraints: BatchingMethod,
batching_deep: BatchingMethod,
) -> ProofOptions {
assert!(num_queries > 0, "number of queries must be greater than 0");
assert!(num_queries <= MAX_NUM_QUERIES, "number of queries cannot be greater than 255");
assert!(blowup_factor.is_power_of_two(), "blowup factor must be a power of 2");
assert!(blowup_factor >= MIN_BLOWUP_FACTOR, "blowup factor cannot be smaller than 2");
assert!(blowup_factor <= MAX_BLOWUP_FACTOR, "blowup factor cannot be greater than 128");
assert!(
grinding_factor <= MAX_GRINDING_FACTOR,
"grinding factor cannot be greater than 32"
);
assert!(fri_folding_factor.is_power_of_two(), "FRI folding factor must be a power of 2");
assert!(
fri_folding_factor >= FRI_MIN_FOLDING_FACTOR,
"FRI folding factor cannot be smaller than 2"
);
assert!(
fri_folding_factor <= FRI_MAX_FOLDING_FACTOR,
"FRI folding factor cannot be greater than 16"
);
assert!(
(fri_remainder_max_degree + 1).is_power_of_two(),
"FRI polynomial remainder degree must be one less than a power of two"
);
assert!(
fri_remainder_max_degree <= FRI_MAX_REMAINDER_DEGREE,
"FRI polynomial remainder degree cannot be greater than 255"
);
Self {
num_queries: num_queries as u8,
blowup_factor: blowup_factor as u8,
grinding_factor: grinding_factor as u8,
field_extension,
fri_folding_factor: fri_folding_factor as u8,
fri_remainder_max_degree: fri_remainder_max_degree as u8,
partition_options: PartitionOptions::new(1, 1),
batching_constraints,
batching_deep,
}
}
@ options.rs 1 ProofOptions::num_queries
pub const fn num_queries(&self) -> usize {
self.num_queries as usize
}
@ options.rs 1 ProofOptions::blowup_factor
pub const fn blowup_factor(&self) -> usize {
self.blowup_factor as usize
}
@ options.rs 1 ProofOptions::grinding_factor
pub const fn grinding_factor(&self) -> u32 {
self.grinding_factor as u32
}
@ options.rs 1 ProofOptions::field_extension
pub const fn field_extension(&self) -> FieldExtension {
self.field_extension
}
@ options.rs 1 ProofOptions::to_fri_options
pub fn to_fri_options(&self) -> FriOptions {
let folding_factor = self.fri_folding_factor as usize;
let remainder_max_degree = self.fri_remainder_max_degree as usize;
FriOptions::new(self.blowup_factor(), folding_factor, remainder_max_degree)
}
@ options.rs 1 ProofOptions::constraint_batching_method
pub fn constraint_batching_method(&self) -> BatchingMethod {
self.batching_constraints
}
@ options.rs 1 ProofOptions::deep_poly_batching_method
pub fn deep_poly_batching_method(&self) -> BatchingMethod {
self.batching_deep
}
@ options.rs 1 FieldExtension::degree
pub const fn degree(&self) -> u32 {
match self {
Self::None => 1,
Self::Quadratic => 2,
Self::Cubic => 3,
}
}
"""


def _cableado(texto):
    """((ruta, veces, funcion, lineas), ...) de CABLEADO."""
    out = []
    for linea in texto.split("\n"):
        if linea.startswith("@ "):
            rel, veces, funcion = linea[2:].split(" ", 2)
            out.append((os.path.join(DIR_FORK, rel), int(veces), funcion, []))
        elif linea:
            out[-1][3].append(linea)
    return tuple((r, v, f, tuple(b)) for r, v, f, b in out)


ANCLAS_CABLEADO = _cableado(CABLEADO)
# Y las dos reglas de longitud que la replica usa, en el fuente que las aplica: la traza oculta de
# una familia de longitud fija tiene 2T filas (la forma que `comprobar_forma` exige), y la de la
# edad 2 * CICLO * 2^m (la que recorre `tests_nivel`). Se cuentan las lineas del fichero ENTERO
# cuyo codigo, sin sangria ni comentario de cola `//`, es la regla; un `/* */` no se distingue, y
# el rojo LONGITUD no da la linea (`:0`).
ANCLAS_LONGITUD = (
    (FUENTE_OPCIONES, "let exige = (ancho + 1, ancho_aux, aleatorios_aux, 2 * longitud);", 1),
    (FUENTE_EDAD, "let filas = 2 * (CICLO << m);", 2),
)
# F.1 b): los KAT del fork, los `assert_eq!` de `mod tests` de security.rs con lote LINEAL en las
# dos fases, el unico que la replica sabe. Valores de la funcion real: el canon corre esos tests en
# el nivel sello (tools/canon.sh:123). CONGELADOS aqui, copiados en el §796: la linea es la de su
# assert hoy, y solo sirve para encontrarlo. (linea, cifra, valor, consultas, blowup, molienda,
# grado de la extension, plegado, resto, log2 de la longitud de la traza)
KAT_FORK = (
    (391, "udr", 100, 119, 4, 20, 2, 2, 127, 20),
    (392, "ldr", 69, 119, 4, 20, 2, 2, 127, 20),
    (416, "ldr", 69, 150, 4, 20, 2, 2, 127, 20),
    (442, "ldr", 100, 81, 4, 20, 3, 2, 127, 20),
    (478, "udr", 116, 123, 8, 16, 2, 2, 7, 8),
    (500, "udr", 115, 123, 8, 16, 2, 4, 7, 8),
    (536, "udr", 100, 195, 2, 20, 2, 2, 7, 8),
    (565, "ldr", 100, 163, 2, 20, 3, 2, 7, 8),
    (591, "udr", 100, 119, 4, 20, 2, 2, 7, 8),
    (616, "ldr", 100, 81, 4, 20, 3, 2, 7, 8),
    (652, "ldr", 99, 80, 4, 20, 3, 8, 127, 18),
    (677, "ldr", 99, 53, 8, 20, 3, 8, 127, 18),
    (713, "ldr", 128, 85, 8, 20, 3, 8, 127, 18),
    (738, "ldr", 128, 65, 16, 20, 3, 8, 127, 18),
    (774, "ldr", 70, 85, 8, 20, 2, 8, 127, 18),
    (799, "ldr", 128, 85, 8, 20, 3, 8, 127, 18),
    (1215, "udr", 108, 120, 8, 20, 2, 2, 255, 16),
    (1335, "ldr", 126, 120, 8, 20, 3, 8, 255, 22),
)
# Los sitios que hoy publican el nivel de PRODUCCION (los que enumera el §698, mas el §708). No es
# un tope: cualquier otro documento vivo se cruza igual. Esta lista existe para que uno que DEJE de
# publicarlo no pase callando.
SITIOS_NIVEL = ("SECURITY.md", "FIVE_BACKENDS.md", "PRINCIPIOS.md", "PAPER.md", "PAPER_EN.md",
                "doc/ZENODO.md", "doc/INSTITUCIONAL.md", "doc/INSTITUTIONAL.md",
                "crates/zk-ssl/src/lib.rs")
# Los .rs del universo: solo sus comentarios. Hoy, el de `proof_options()`.
FUENTES_RS_NIVEL = ("crates/zk-ssl/src/lib.rs",)
EXCLUIDOS_NIVEL = {
    "doc/blueprint-v2.md": "el debate del plan: cita niveles de OTRAS opciones -q = 33, 34 y 46, "
                           "UDR 52 o 49- como propuestas, no lo que se publica",
    "doc/ecst/VERIFICACION.md": "registro generado sobre el arbol de 71c5aad (S582): cada fila "
                                "cita sus fuentes con el texto de esa fecha, y envejece declarado",
}
PREFIJOS_EXCLUIDOS_NIVEL = {
    "doc/comparativa/": "comparativas superadas por FIVE_BACKENDS.md, conservadas como historia "
                        "(§448)",
    "doc/historia/": "historia: no se reescribe",
}
# Las que el universo hereda del ATADO C, con la razon de ESTE atado. Tienen que ser las mismas que
# EXCLUIDOS_URL y PREFIJOS_EXCLUIDOS_URL: si el C cambia las suyas y estas no, es ROJO.
HEREDADOS_NIVEL = {
    "AUDITORIA.md": "registro historico: un asiento cita el nivel de su fecha",
    "BACKLOG.md": "registro: las entradas no se reescriben (la 116 cita lo medido en el §697)",
    "doc/preprints/": "entrada 28, SUSPENDIDA hasta el fin del proyecto: el preprint habla de la "
                      "brecha entre conjeturada y demostrable sin cifra que este atado lea "
                      "(ZK-SSL-preprint.md:234-236); lo que alli quede rancio se DECLARA",
}
# Cifras de OTRO sistema: (fichero, ancla de texto en la frase, razon).
AJENAS_NIVEL = (
    ("FIVE_BACKENDS.md", "98 bits conjeturados",
     "RISC Zero: la que declara su documentacion, no sale de ninguna constante del arbol"),
)
# Lo medido al nacer el atado, en el §796: 154 cifras en 51 frases, y 68 celdas -12 de la tabla de
# longitud fija, 44 de las dos de la edad y 12 de la de la comparacion-. Es un suelo, no una
# cuenta: crecer no lo toca, y encoger es un rojo que obliga a bajarlo con su asiento.
MINIMO_CIFRAS_F = 154
MINIMO_CELDAS_F = 68
# Las marcas que eligen configuracion. «ocult-» precedido de «sin» o «ni» es de la comparacion.
MARCA_PRODUCCION = re.compile(
    r"producci[oó]n|\bproduction\b|(?<!sin )(?<!ni )\bocult[ao]s?\b|\bhidden\b|proof_options|"
    r"\b42 (?:consultas|queries)\b|opciones de la capa|layer's",
    re.I,
)
MARCA_COMPARACION = re.compile(
    r"comparaci[oó]n|\bcomparison\b|circuito de cumplimiento|compliance circuit|"
    r"\b32 (?:consultas|queries)\b|sin molienda|sin grinding|no grinding|sin ocultar|"
    r"ni ocultaci[oó]n|without hiding",
    re.I,
)
_N = r"(?<![\w,.~])~?(\d{1,3})(?![\d,.])"
_D = r"(?<![\w,.])(\d{2,3}[,.]\d)(?![\d])"
_DE_UDR = r"\s*(?:\(UDR\)|(?:en|under|in)\s+UDR\b)"
_DE_LDR = r"\s*(?:\(LDR\)|(?:en|under|in)\s+LDR\b)"
_RANGO = r"\s*(?:[-–−]|\s+a\s+|\s+to\s+)\s*"
# (clase, patron). El orden importa: un tramo ya leido por un patron no lo lee el siguiente.
PATRONES_NIVEL = (
    ("ldr_m", re.compile(_N + r"\s+(?:bits\s+)?(?:con|with)\s+`?m\s*=\s*(\d+)`?")),
    ("deep_m", re.compile(_D + r"\s+(?:bits\s+)?(?:en\s+la\s+edad\s+|in\s+the\s+age\s+proof\s+)?"
                          r"(?:con|with)\s+`?m\s*=\s*(\d+)`?")),
    ("deep_rango", re.compile(r"(?:\bde\s+)?" + _D + _RANGO + r"(\d{2,3}[,.]\d)(?![\d])")),
    ("deep_L", re.compile(_D + r"\s+(?:bits\s+)?(?:con|with)\s+(?:las\s+|the\s+)?"
                          r"(\d{1,3}(?:\.\d{3})+|\d{2,9})(?!\d|,\d)(?!\s*(?:KB|B\b|ms))")),
    ("ldr_min", re.compile(r"\bLDR\s*(?:≥|>=)\s*(\d{1,3})")),
    ("conj_min", re.compile(r"\b(?:[Cc]onjeturada|[Cc]onjectured)\s*(?:≥|>=)\s*(\d{1,3})")),
    ("par", re.compile(_N + r"\s*[-–/]\s*(\d{1,3})\s+(?:bits\s+)?(?:of\s+)?"
                       r"(?:demostrables|provable)")),
    ("ldr_rango", re.compile(r"(?:\bde\s+)?" + _N + _RANGO + r"(\d{1,3})" + _DE_LDR)),
    ("ldr_rango", re.compile(r"\bLDR\s*=?\s*(\d{1,3})\s*[-–−]\s*(\d{1,3})(?![\d,.])")),
    ("ldr", re.compile(_N + _DE_LDR)),
    ("ldr", re.compile(r"\bLDR\s*=?\s*(\d{1,3})(?![\d,.])")),
    ("udr", re.compile(_N + _DE_UDR)),
    ("udr", re.compile(r"\bUDR\s*=?\s*(\d{1,3})(?![\d,.])")),
    ("conj", re.compile(_N + r"\s+(?:bits\s+)?(?:(?:de\s+seguridad|of|declarados\s+son|are)\s+)?"
                        r"(?:conjeturad[oa]s?|conjectured)\b")),
    ("conj", re.compile(r"\b(?:[Cc]onjeturada|[Cc]onjectured)\s+(\d{2,3})(?![\d,.])")),
    ("bits_consultas", re.compile(r"(?:consultas|queries)\s+(?:dan|give)\s+(\d{2,3})\s+bits")),
    ("q", re.compile(_N + r"\s+(?:consultas|queries)\b")),
    ("blowup", re.compile(r"\bblowup\s+(?:de\s+)?(\d{1,3})\b")),
    ("molienda", re.compile(r"\b(?:molienda|grinding)\s+(?:de\s+)?(\d{1,2})\b")),
    ("molienda0", re.compile(r"\bsin\s+(?:molienda|grinding)|\bno\s+grinding", re.I)),
    ("ext2", re.compile(r"extensi[oó]n\s+cuadr[aá]tica|ext\.\s*cuadr[aá]tica|"
                        r"quadratic\s+extension", re.I)),
    ("ext3", re.compile(r"extensi[oó]n\s+c[uú]bica|ext\.\s*c[uú]bica|cubic\s+extension", re.I)),
    ("ext1", re.compile(r"sin\s+extensi[oó]n|without\s+(?:a\s+)?field\s+extension|"
                        r"\bno\s+(?:field\s+)?extension", re.I)),
)
# Las clases de NIVEL, que una fila de la comparacion tambien da, y las que solo da produccion.
CLASES_DE_NIVEL = ("conj", "udr", "ldr", "par")
SOLO_PRODUCCION = ("ldr_m", "deep_m", "deep_rango", "deep_L", "ldr_min", "conj_min", "ldr_rango",
                   "bits_consultas")
CABEZA_FIJAS = "| traza oculta | LDR | conjeturada, con el DEEP | pruebas |"
CABEZA_COMPARACION = "| Configuración | Tamaño | Generación | Conjeturada | Demostrable |"



def constante():
    """Lee la fuente unica. Por ESTRUCTURA, no por posicion."""
    try:
        with open(FUENTE, encoding="utf-8") as fh:
            src = fh.read()
    except OSError as exc:
        print("ROJO: no se puede leer la fuente unica: %s" % exc)
        sys.exit(1)
    # S538 (RFC-0009 D-AC): la cifra es una BANDA con dos extremos publicos; una cita
    # vale si es uno de los dos, en MiB (2^20) o en su equivalente SI (10^6).
    ext = {}
    for nombre in ("PUBLICADA_PAGO_MIN_B", "PUBLICADA_PAGO_MAX_B"):
        m = re.search(r"const %s:\s*usize\s*=\s*([0-9_]+)" % nombre, src)
        if not m:
            print("ROJO: no aparece %s en %s" % (nombre, FUENTE))
            print("      la fuente unica son esas dos constantes: sin ellas no hay atado")
            sys.exit(1)
        ext[nombre] = int(m.group(1).replace("_", ""))
    b = (ext["PUBLICADA_PAGO_MIN_B"], ext["PUBLICADA_PAGO_MAX_B"])
    f = re.search(r'const PUBLICADA_FECHA:\s*&str\s*=\s*"([^"]+)"', src)
    mib = tuple("%.1f" % (x * 1000 / 1048576.0) for x in b)
    msi = tuple("%.1f" % (x * 1000 / 1000000.0) for x in b)
    return b, mib, msi, (f.group(1) if f else "sin fecha")


# ---- ATADO B, las filas (§841) ----

# (documento, comienzo de la fila, banda, separador de miles). La banda: ENVIO y COBRO son
# BANDA_ENVIO_B y BANDA_COBRO_B; PAGO, PUBLICADA_PAGO_MIN_B y PUBLICADA_PAGO_MAX_B; UNION, la
# menor de las dos cotas bajas y la mayor de las dos altas.
FILAS_BANDA = (
    ("ARQUITECTURA.md", "| Prueba de liquidación (envío o cobro) |", "UNION", "."),
    ("ARQUITECTURA.md", "| Transferencia (envío + cobro) |", "PAGO", "."),
    ("ARQUITECTURA.md", "| ├─ envío |", "ENVIO", "."),
    ("ARQUITECTURA.md", "| └─ cobro |", "COBRO", "."),
    ("PAPER.md", "| Transferencia (envío + cobro) |", "PAGO", "."),
    ("PAPER.md", "| ├─ envío |", "ENVIO", "."),
    ("PAPER.md", "| └─ cobro |", "COBRO", "."),
    ("PAPER_EN.md", "| Transfer (send + claim) |", "PAGO", ","),
    ("PAPER_EN.md", "| ├─ send |", "ENVIO", ","),
    ("PAPER_EN.md", "| └─ claim |", "COBRO", ","),
)
BANDA_EN_CELDA = re.compile(r"(?<![\d.,])(\d{1,3}(?:[.,]\d{3})+)\s*[–-]\s*(\d{1,3}(?:[.,]\d{3})+)\s*B\b")


def bandas(pago_b):
    """Las bandas de metrics.rs, leidas por su nombre: {clase: (min, max)}."""
    with open(FUENTE, encoding="utf-8") as fh:
        src = fh.read()
    out = {"PAGO": tuple(pago_b)}
    for clase, nombre in (("ENVIO", "BANDA_ENVIO_B"), ("COBRO", "BANDA_COBRO_B")):
        m = re.search(r"const %s:\s*\(usize,\s*usize\)\s*=\s*\(\s*([0-9_]+)\s*,\s*([0-9_]+)\s*\)"
                      % nombre, src)
        if not m:
            return None, "no aparece %s en %s" % (nombre, FUENTE)
        out[clase] = (int(m.group(1).replace("_", "")), int(m.group(2).replace("_", "")))
    out["UNION"] = (min(out["ENVIO"][0], out["COBRO"][0]), max(out["ENVIO"][1], out["COBRO"][1]))
    return out, None


def _miles(n, sep):
    return "{:,}".format(n).replace(",", sep)


def atado_b_filas(pago_b):
    """Cada fila de FILAS_BANDA, una vez en su documento y con su banda en la ultima celda.
    Devuelve (fallos, filas vistas)."""
    bs, error = bandas(pago_b)
    if bs is None:
        return [("metrics.rs", 0, "BANDA", error)], 0
    fallos, vistas = [], 0
    for rel, cabeza, clase, sep in FILAS_BANDA:
        try:
            with open(os.path.join(RAIZ, rel), encoding="utf-8") as fh:
                lineas = fh.read().split("\n")
        except OSError as exc:
            fallos.append((rel, 0, "ILEGIBLE", str(exc)))
            continue
        donde = [(i, l) for i, l in enumerate(lineas, 1) if l.startswith(cabeza)]
        if len(donde) != 1:
            fallos.append((rel, 0, "FILA", "la fila «%s» esta %d vez/veces; tiene que estar una"
                           % (cabeza, len(donde))))
            continue
        n, linea = donde[0]
        vistas += 1
        celdas = [c.strip() for c in linea.strip().strip("|").split("|")]
        hay = [(int(a.replace(sep, "")), int(b.replace(sep, ""))) if sep in a + b else None
               for a, b in BANDA_EN_CELDA.findall(celdas[-1])]
        debe = bs[clase]
        if hay != [debe]:
            fallos.append((rel, n, "BANDA", "la fila «%s» da %s; la banda %s es %s–%s B"
                           % (cabeza, celdas[-1], clase, _miles(debe[0], sep),
                              _miles(debe[1], sep))))
    return fallos, vistas


# ---- RETIRADAS (§841) ----

RETIRADAS = (
    ("1,12 ms", re.compile(r"(?<![\d.,])1[.,]12\s*ms\b")),
    ("0,67 ms", re.compile(r"(?<![\d.,])0[.,]67\s*ms\b")),
    ("61.966 B", re.compile(r"(?<![\d.,])61[.,]966\s*B\b")),
    ("125,6 KB", re.compile(r"(?<![\d.,])125[.,]6\s*KB\b")),
)
MARCA_RETIRADA = re.compile(r"\bdec[ií]an?\b|\bit said\b|\u26a0")
EXCLUIDOS_RETIRADAS = {
    "doc/blueprint-v2.md": "el debate del plan: su tabla contrasta lo publicado en su fecha con "
                           "lo medido en la sesion, y lo dice",
    "doc/ecst/VERIFICACION.md": "registro generado sobre el arbol de 71c5aad (S582): cada fila "
                                "cita sus fuentes con el texto de esa fecha",
}
PREFIJOS_EXCLUIDOS_RETIRADAS = {
    "doc/comparativa/": "comparativas superadas por FIVE_BACKENDS.md, conservadas como historia "
                        "(§448)",
    "doc/historia/": "historia: no se reescribe",
}
# (fichero, ancla en la frase, razon): frases que citan una cifra retirada como lo que otro
# decia, sin la palabra. Cada ancla tiene que casar al menos una vez.
CITAS_RETIRADAS = (
    ("SECURITY.md", "que\npublicaban varios documentos hasta el §698",
     "cita lo que otros documentos publicaban hasta el §698, y lo dice"),
    ("FIVE_BACKENDS.md", "que imprime el\ntest son sus 27,7, 32,7, 36,7 y 125,6 KB",
     "explica la remedida del §806: lo que el arbol de 9a1e2f1 imprimia en KB"),
    ("doc/integracion-vertical-evaluacion.md", "la casa declara 127 conjeturados, y 128 "
     "demostrables costarían «125,6 KB en vez de 36,7»",
     "evaluacion fechada (01-10-2026, aceptada en el §705): cita lo que la casa publicaba "
     "entonces, y su columna de correccion dice que no era produccion"),
    ("doc/integracion-vertical-evaluacion.md", "La casa declara **127 bits conjeturados** y que "
     "128 **demostrables** costarían 125,6 KB por prueba en vez de 36,7",
     "evaluacion fechada (01-10-2026, aceptada en el §705): cita lo que la casa publicaba "
     "entonces"),
)
# Lo medido al nacer, en el §841: las citas marcadas que el universo da. Es un suelo, como los
# del ATADO F: encoger es un rojo que obliga a bajarlo con su asiento.
MINIMO_RETIRADAS = 20


def documentos_retiradas():
    """El universo de RETIRADAS: los .md del ATADO C menos sus exclusiones, y los .rs de crates/."""
    for rel in documentos_url():
        if not rel.endswith(".md") or rel in EXCLUIDOS_RETIRADAS:
            continue
        if rel.startswith(tuple(PREFIJOS_EXCLUIDOS_RETIRADAS)):
            continue
        yield rel
    rs = []
    for base, dirs, fs in os.walk(os.path.join(RAIZ, "crates")):
        dirs[:] = [d for d in dirs if d != "target"]
        rs.extend(os.path.relpath(os.path.join(base, f), RAIZ) for f in fs if f.endswith(".rs"))
    for rel in sorted(rs):
        yield rel


def retiradas():
    """Devuelve (fallos, citas marcadas, citas por excepcion, ficheros recorridos)."""
    anclas = [(rel, " ".join(_limpia(a).split()), razon, [0]) for rel, a, razon in CITAS_RETIRADAS]
    fallos, marcadas, por_excepcion, recorridos = [], 0, 0, 0
    for rel in documentos_retiradas():
        recorridos += 1
        for texto, mapa, es_fila, _ in _unidades(rel):
            if not texto:
                continue
            for etiqueta, patron in RETIRADAS:
                for m in patron.finditer(texto):
                    p = m.start()
                    a, b = next(((a, b) for a, b in _tramos(texto, es_fila) if a <= p < b),
                                (0, len(texto)))
                    ini = max([o for o, _ in mapa if o <= p] or [0])
                    if MARCA_RETIRADA.search(texto[a:p]) or MARCA_RETIRADA.search(texto[ini:p]):
                        marcadas += 1
                        continue
                    frase = " ".join(texto[a:b].split())
                    casa = [x for x in anclas if x[0] == rel and x[1] in frase]
                    if casa:
                        for x in casa:
                            x[3][0] += 1
                        por_excepcion += 1
                        continue
                    fallos.append((rel, _linea_de(mapa, p), "RETIRADA",
                                   "«%s» sin «decía», «it said» ni ⚠️ antes, en su línea o su "
                                   "frase" % etiqueta))
    for rel, ancla, _, usos in anclas:
        if not usos[0]:
            fallos.append((rel, 0, "EXCEPCION", "el ancla «%s» ya no casa con ninguna cifra "
                           "retirada: quita la excepcion" % ancla[:60]))
    return fallos, marcadas, por_excepcion, recorridos


def documentos():
    for nombre in sorted(os.listdir(RAIZ)):
        if nombre.endswith(".md") and nombre not in EXCLUIDOS:
            yield nombre



def documentos_url():
    """Universo del ATADO C: todo .md bajo la raiz, recursivo, menos los registros
    y doc/preprints/; mas CITATION.cff, que no es .md y es lo primero que lee
    quien llega de fuera."""
    for base, dirs, fs in os.walk(RAIZ):
        dirs[:] = [d for d in dirs if d not in {".git", "target", ".canon"}]
        for f in sorted(fs):
            if not f.endswith(".md"):
                continue
            rel = os.path.relpath(os.path.join(base, f), RAIZ)
            if rel in EXCLUIDOS_URL or rel.startswith(PREFIJOS_EXCLUIDOS_URL):
                continue
            yield rel
    yield "CITATION.cff"


def atado_c():
    """Devuelve (fallos, citas de la URL nueva, documentos recorridos)."""
    fallos = []
    nuevas = 0
    recorridos = 0
    for rel in documentos_url():
        ruta = os.path.join(RAIZ, rel)
        try:
            with open(ruta, encoding="utf-8") as fh:
                lineas = fh.readlines()
        except OSError as exc:
            fallos.append((rel, 0, "ILEGIBLE", str(exc)))
            continue
        recorridos += 1
        for i, linea in enumerate(lineas, 1):
            if TOKEN_VIEJO in linea:
                fallos.append((rel, i, "URL VIEJA", linea.strip()[:70]))
            if URL_REPO in linea:
                nuevas += 1
    try:
        with open(CFF, encoding="utf-8") as fh:
            cff = fh.read()
    except OSError as exc:
        fallos.append(("CITATION.cff", 0, "ILEGIBLE", str(exc)))
        return fallos, nuevas, recorridos
    m = re.search(r'^repository-code:\s*"([^"]+)"', cff, re.M)
    if not m:
        fallos.append(("CITATION.cff", 0, "SIN repository-code", ""))
    elif m.group(1) != URL_REPO:
        fallos.append(("CITATION.cff", 0, "repository-code", m.group(1)))
    return fallos, nuevas, recorridos


def rfcs():
    """(numero, ruta, estado) de cada RFC del directorio, menos la plantilla.
    El PRODUCTOR del estado es la linea `- **Estado:**` del propio RFC: una sola."""
    salida = []
    for f in sorted(os.listdir(DIR_RFC)):
        m = re.match(r"^(\d{4})-.*\.md$", f)
        if not m or m.group(1) == PLANTILLA:
            continue
        with open(os.path.join(DIR_RFC, f), encoding="utf-8") as fh:
            txt = fh.read()
        e = re.findall(r"^- \*\*Estado:\*\* ([A-Z]+)", txt, re.M)
        salida.append((m.group(1), "spec/rfc/" + f, e))
    return salida


def repartir(linea):
    """{numero: ESTADO} de una linea que ENUMERA: los numeros que van antes de una
    palabra de estado son suyos, desde la palabra anterior. Devuelve tambien los
    numeros repetidos, porque un diccionario se queda con el ultimo y no lo diria."""
    grupos = {}
    repes = [n for n in set(NUM_RFC.findall(linea))
             if NUM_RFC.findall(linea).count(n) > 1]
    ini = 0
    for m in PAL_ESTADO.finditer(linea):
        pal = m.group(0).lower()
        estado = None
        for e, (_, palabras) in ESTADOS.items():
            if pal in palabras:
                estado = e
        if estado is not None:
            for num in NUM_RFC.findall(linea[ini:m.start()]):
                grupos[num] = estado
        ini = m.end()
    return grupos, sorted(repes)


def atado_d():
    """Devuelve (fallos, RFC con estado, sitios con cuenta, cuentas halladas)."""
    fallos = []
    lista = rfcs()
    conocidos = set(num for num, _, _ in lista)
    esperado = {}
    for num, rel, estados in lista:
        if len(estados) != 1:
            fallos.append((rel, 0, "ESTADO", "declara %d lineas de estado, se esperaba 1"
                           % len(estados)))
        elif estados[0] not in ESTADOS:
            fallos.append((rel, 0, "ESTADO", "%s no tiene cara publicada; el vocabulario "
                           "vive en spec/rfc/PROCESO.md" % estados[0]))
        else:
            esperado[num] = estados[0]

    # D.0 - el OPERADOR contra el PRODUCTOR. `rfcs()` lee el numero del NOMBRE del fichero
    # con cuatro digitos; `repartir()` lo lee de una linea de prosa con NUM_RFC. Si el
    # segundo no sabe leer lo que el primero produce, el gate se queda CIEGO con ese RFC y
    # no lo dice: paso con el 0010 y `000\d` (S555). Un techo callado, nunca mas.
    for num, rel, _ in lista:
        if not NUM_RFC.fullmatch(num):
            fallos.append((rel, 0, "OPERADOR", "NUM_RFC no sabe leer el RFC-%s: mientras "
                           "no lo lea, ninguna cuenta puede citarlo" % num))

    # D.1 - la FILA de spec/README.md: una por RFC, y con su token de estado
    with open(os.path.join(RAIZ, FILA_RFC), encoding="utf-8") as fh:
        filas = fh.read().split("\n")
    for num, rel, _ in lista:
        if num not in esperado:
            continue
        cabeza = "| `" + rel[len("spec/"):] + "`"
        hit = [(i + 1, l) for i, l in enumerate(filas) if l.startswith(cabeza)]
        if len(hit) != 1:
            fallos.append((FILA_RFC, 0, "FILA", "el RFC-%s tiene %d filas, se esperaba 1"
                           % (num, len(hit))))
            continue
        n, linea = hit[0]
        token = "RFC-%s, %s" % (num, ESTADOS[esperado[num]][0])
        if token not in linea:
            dice = re.search(r"RFC-%s[,:]?\s*[A-Z]*" % num, linea)
            fallos.append((FILA_RFC, n, "TOKEN", "no dice <<%s>>; dice <<%s>>"
                           % (token, dice.group(0).strip() if dice else "nada")))

    # D.2 - los CARDINALES: toda linea del universo con tres numeros y una palabra de
    # estado es una cuenta, y una cuenta las lleva TODAS, cada una en su grupo.
    sitios = {}
    for rel in documentos_url():
        if rel == FILA_RFC or not rel.endswith(".md"):
            continue
        try:
            with open(os.path.join(RAIZ, rel), encoding="utf-8") as fh:
                lineas = fh.read().split("\n")
        except OSError as exc:
            fallos.append((rel, 0, "ILEGIBLE", str(exc)))
            continue
        for i, linea in enumerate(lineas, 1):
            if not PAL_ESTADO.search(linea):
                continue
            if len(set(NUM_RFC.findall(linea))) < UMBRAL_CARDINAL:
                continue
            sitios.setdefault(rel, []).append(i)
            if RANGO_RFC.search(linea):
                fallos.append((rel, i, "RANGO", "una cuenta se ENUMERA: un rango lo lee "
                               "mal quien lo lee y quien lo mide"))
                continue
            grupos, repes = repartir(linea)
            for num in repes:
                fallos.append((rel, i, "DUPLICADO", "el RFC-%s sale dos veces" % num))
            for num in sorted(esperado):
                if num not in grupos:
                    fallos.append((rel, i, "AUSENTE", "el RFC-%s no esta en la cuenta" % num))
                elif grupos[num] != esperado[num]:
                    fallos.append((rel, i, "DISCREPA", "el RFC-%s sale como %s y su RFC dice "
                                   "%s" % (num, grupos[num], esperado[num])))
            for num in sorted(set(grupos) - set(esperado)):
                if num in conocidos:
                    continue   # su RFC ya tiene su ROJO arriba: una causa, un rojo
                fallos.append((rel, i, "SOBRA", "cita el RFC-%s, que no esta en spec/rfc/"
                               % num))
    for nombre in CARDINALES:
        if nombre not in sitios:
            fallos.append((nombre, 0, "SIN CUENTA", "sitio conocido que ha dejado de "
                           "publicar la cuenta de los RFC"))
    return fallos, len(esperado), len(sitios), sum(len(v) for v in sitios.values())


def atado_e():
    """Devuelve (fallos, ficheros contados, citas halladas, sitios que la citan)."""
    n = sum(len(fs) for _, _, fs in os.walk(DIR_VECTORES))
    fallos, citas, sitios = [], 0, set()
    for nombre in SITIOS_VECTORES:
        with open(os.path.join(RAIZ, nombre), encoding="utf-8") as fh:
            for i, linea in enumerate(fh, 1):
                if "spec/vectors/" not in linea:
                    continue
                for m in CUENTA_VECTORES.finditer(linea):
                    citas += 1
                    sitios.add(nombre)
                    if int(m.group(1)) != n:
                        fallos.append((nombre, i, "VECTORES", "dice %s y spec/vectors/ tiene %d "
                                       "ficheros" % (m.group(1), n)))
    for nombre in SITIOS_VECTORES:
        if nombre not in sitios:
            fallos.append((nombre, 0, "VECTORES", "ya no publica la cuenta de spec/vectors/: si "
                           "es a proposito, sale de SITIOS_VECTORES con su razon"))
    return fallos, n, citas


def _cabecera(ruta):
    """Las lineas antes del primer `## `, numeradas desde 1."""
    salida = []
    with open(ruta, encoding="utf-8") as fh:
        for i, linea in enumerate(fh.read().split("\n"), 1):
            if linea.startswith("## "):
                break
            salida.append((i, linea))
    return salida


def _vinetas(cabecera):
    """([(linea, nombre, plural)] de las vinetas de autor, [(linea, texto)] de las de GenAI, con las
    lineas de continuacion unidas a la suya)."""
    autores, genai = [], []
    for k, (i, linea) in enumerate(cabecera):
        m = AUTOR_RFC.match(linea)
        if m:
            autores.append((i, m.group(2).strip(), bool(m.group(1))))
        m = VINETA_GENAI.match(linea)
        if m:
            partes = [m.group(1)]
            for _, sig in cabecera[k + 1:]:
                if not sig.strip() or not sig[0].isspace():
                    break
                partes.append(sig.strip())
            genai.append((i, " ".join(partes).strip()))
    return autores, genai


def _problemas_g(autores, genai):
    """Lo que le falta o le sobra a una cabecera de RFC: [(linea, clase, detalle)]."""
    salida = []
    if len(autores) != 1:
        salida.append((0, "AUTOR", "%d vineta(s) de autor; se espera una «- **Autor:** <nombre>»"
                       % len(autores)))
    elif autores[0][2]:
        salida.append((autores[0][0], "AUTOR", "dice «Autores»; el autor es uno, «- **Autor:**»"))
    elif autores[0][1] not in NOMBRES_AUTOR:
        salida.append((autores[0][0], "AUTOR", "dice «%s»; el autor es uno de %s (GENAI.md)"
                       % (autores[0][1][:50], ", ".join("«%s»" % n for n in NOMBRES_AUTOR))))
    if len(genai) != 1:
        salida.append((0, "GENAI", "%d vineta(s) «- **Asistencia GenAI:**»; se espera una"
                       % len(genai)))
    for i, texto in genai:
        if not texto:
            salida.append((i, "GENAI", "la vineta de GenAI esta vacia"))
        m = MODELO.search(texto)
        if m:
            salida.append((i, "MODELO", "la vineta de GenAI nombra un identificador de modelo "
                           "(«%s…»); la formula de la casa es «Claude, en una sesion de Claude "
                           "Code en la nube»" % m.group(0)))
    return salida


def _asiento(texto, numero):
    """(linea donde empieza, cuerpo) del asiento `## <numero> —` de AUDITORIA.md, o None."""
    lineas = texto.split("\n")
    ini = next((k for k, l in enumerate(lineas) if l.startswith("## %s " % numero)), None)
    if ini is None:
        return None
    fin = next((k for k in range(ini + 1, len(lineas)) if lineas[k].startswith("## ")),
               len(lineas))
    return ini + 1, lineas[ini:fin]


def atado_g():
    """Devuelve (fallos, RFC vistos, RFC con sus dos vinetas, excepciones con su cita, si esta la
    plantilla)."""
    fallos, vistos, completos, citas, plantilla = [], 0, 0, [], False
    try:
        with open(GENAI, encoding="utf-8") as fh:
            genai_md = " ".join(fh.read().split())
    except OSError as exc:
        return [("GENAI.md", 0, "ILEGIBLE", str(exc))], 0, 0, [], False
    for nombre in NOMBRES_AUTOR:
        if nombre not in genai_md:
            fallos.append(("GENAI.md", 0, "NOMBRE", "no da «%s», y NOMBRES_AUTOR lo acepta: la "
                           "lista sale de GENAI.md" % nombre))
    try:
        with open(AUDITORIA, encoding="utf-8") as fh:
            auditoria = fh.read()
    except OSError as exc:
        auditoria = None
        fallos.append(("AUDITORIA.md", 0, "ILEGIBLE", str(exc)))
    vistos_num = set()
    for f in sorted(os.listdir(DIR_RFC)):
        m = re.match(r"^(\d{4})-.*\.md$", f)
        if not m:
            continue
        num, rel = m.group(1), "spec/rfc/" + f
        cab = _cabecera(os.path.join(DIR_RFC, f))
        autores, genai = _vinetas(cab)
        if num == PLANTILLA:
            plantilla = True
            # La forma de plantilla: una vineta de autor con un hueco `<...>` y una de GenAI.
            if len(autores) != 1 or not re.fullmatch(r"<[^<>]+>", autores[0][1]) \
                    or len(genai) != 1:
                fallos.append((rel, 0, "PLANTILLA", "la plantilla no da la forma: una «- "
                               "**Autor:** <...>» y una «- **Asistencia GenAI:**»"))
            continue
        vistos += 1
        vistos_num.add(num)
        problemas = _problemas_g(autores, genai)
        if num not in EXCEPCIONES_G:
            if problemas:
                fallos.extend((rel, i, clase, detalle) for i, clase, detalle in problemas)
            else:
                completos += 1
            continue
        asiento, frase, linea_autor = EXCEPCIONES_G[num]
        if not problemas:
            fallos.append((rel, 0, "EXCEPCION", "la cabecera ya lleva autor y vineta de GenAI: "
                           "la excepcion esta rancia, sale de EXCEPCIONES_G"))
        elif [l for _, l in cab if AUTOR_RFC.match(l)] != [linea_autor] or genai:
            fallos.append((rel, 0, "EXCEPCION", "la excepcion declara «%s» y ninguna vineta de "
                           "GenAI, y la cabecera ya no es esa: se decide otra vez" % linea_autor))
        if auditoria is None:
            continue
        cuerpo = _asiento(auditoria, asiento)
        if cuerpo is None:
            fallos.append(("AUDITORIA.md", 0, "EXCEPCION", "la excepcion del RFC-%s cita el %s, "
                           "que no esta" % (num, asiento)))
            continue
        ini, lineas = cuerpo
        plano = " ".join(" ".join(lineas).split())
        if frase not in plano:
            fallos.append(("AUDITORIA.md", ini, "EXCEPCION", "el %s ya no dice «%s»: la "
                           "excepcion del RFC-%s se queda sin su razon" % (asiento, frase, num)))
            continue
        primera = frase.split()[:3]
        donde = next((ini + k for k, l in enumerate(lineas) if " ".join(primera) in l), ini)
        citas.append((num, asiento, donde))
    for num in sorted(set(EXCEPCIONES_G) - vistos_num):
        fallos.append(("spec/rfc/", 0, "EXCEPCION", "EXCEPCIONES_G declara el RFC-%s, que no "
                       "esta" % num))
    if not plantilla:
        fallos.append(("spec/rfc/", 0, "PLANTILLA", "no esta la plantilla %s-*.md" % PLANTILLA))
    if vistos < MINIMO_RFC_G:
        fallos.append(("spec/rfc/", 0, "VIDA", "%d RFC, y el minimo medido es %d: un censo que "
                       "encoge se declara" % (vistos, MINIMO_RFC_G)))
    return fallos, vistos, completos, citas, plantilla


# ---- ATADO F: F.0, las constantes ----

def _leer(ruta):
    with open(ruta, encoding="utf-8") as fh:
        return fh.read()


def _args_opciones(texto):
    """Los ocho argumentos de `ProofOptions::new`, o None si no son ocho que se sepan leer."""
    a = [x.strip() for x in texto.split(",") if x.strip()]
    if len(a) != 8:
        return None
    try:
        return {"q": int(a[0]), "blowup": int(a[1]), "molienda": int(a[2]),
                "ext": GRADO_EXTENSION[a[3].split("::")[-1]], "plegado": int(a[4]),
                "resto": int(a[5]), "lotes": (a[6].split("::")[-1], a[7].split("::")[-1])}
    except (ValueError, KeyError):
        return None


def _opciones_de(fuente, nombre):
    m = re.search(r"fn %s\(\) -> ProofOptions \{\s*ProofOptions::new\((.*?)\)\s*\}" % nombre,
                  fuente, re.S)
    return _args_opciones(m.group(1)) if m else None


def _trace_length(modulo):
    """El TRACE_LENGTH de un circuito de stark-experiment, resolviendo las constantes de su
    fichero: el de la apertura del reembolso es `CICLOS * CYCLE_LENGTH`, no un literal."""
    src = _leer(os.path.join(DIR_CIRCUITOS, modulo + ".rs"))
    consts = dict(re.findall(r"^\s*(?:pub(?:\(\w+\))?\s+)?const (\w+): usize = ([^;]+);", src,
                             re.M))

    def valor(nombre, prof):
        if prof > 8 or nombre not in consts:
            raise ValueError("%s no se resuelve en %s.rs" % (nombre, modulo))
        expr = re.sub(r"(?<=\d)_(?=\d)", "", consts[nombre])
        expr = re.sub(r"\b[A-Z][A-Z0-9_]*\b", lambda x: str(valor(x.group(0), prof + 1)), expr)
        if not re.fullmatch(r"[\d\s*+()-]+", expr):
            raise ValueError("%s = %s no es aritmetica de enteros" % (nombre, consts[nombre]))
        return int(eval(expr, {"__builtins__": {}}))   # solo digitos y operadores: lo exige arriba

    return valor("TRACE_LENGTH", 0)


def _pares(src, nombre):
    """(longitud declarada, [(m, valor)]) de una `const NOMBRE: [(u32, u32); N]`."""
    m = re.search(r"const %s: \[\(u32, u32\); (\d+)\] = \[(.*?)\];" % nombre, src, re.S)
    if not m:
        return None, []
    return int(m.group(1)), [(int(a), int(b)) for a, b in re.findall(r"\((\d+),\s*(\d+)\)",
                                                                      m.group(2))]


def _u32(src, nombre):
    return [int(x) for x in re.findall(r"const %s: u32 = (\d+);" % nombre, src)]


# ---- ATADO F: F.1 c), las anclas en el fork ----

def _codigo(src):
    """[(linea, texto)] de cada linea de CODIGO de un fuente de Rust HASTA `mod tests`: se para en
    la primera `mod tests {` que sigue a `#[cfg(test)]`, y lo de detras no se lee. Sin su sangria, y
    sin las lineas blancas ni las que empiezan por `//`; lo de dentro de un `/* */` cuenta."""
    out = []
    for i, linea in enumerate(src.split("\n"), 1):
        s = linea.strip()
        if s == "mod tests {" and out and out[-1][1] == "#[cfg(test)]":
            out.pop()
            break
        if s and not s.startswith("//"):
            out.append((i, s))
    return out


REMEDIO_FORMULA = ("si cambio la formula, cambia la replica (y KAT_FORK, si cambian los "
                   "asserts); si solo su forma (rustfmt, migracion), el ancla se actualiza en el "
                   "MISMO sello, con los tests de Rust del canon en verde "
                   "(tools/canon.sh:122-123)")


def _donde(codigo, bloque, sitios, veces):
    """(linea, que) de un bloque del cableado que no sale sus veces: si sobra, la de la vez de
    mas; si falta, la primera que se aparta en el sitio donde mas lineas casa desde su primera, o,
    si ningun sitio casa ni esa, la de su ultima vez entera o donde esta la mas parecida a ella.
    0 si no hay ni eso."""
    if len(sitios) > veces:
        return codigo[sitios[veces]][0], "; la de mas, aqui"
    textos = [t for _, t in codigo]
    mejor, casan = None, 0
    for i, t in enumerate(textos):
        k = 0
        while k < len(bloque) and i + k < len(textos) and textos[i + k] == bloque[k]:
            k += 1
        if casan < k < len(bloque):
            mejor, casan = i, k
    if mejor is None and sitios:
        return codigo[sitios[-1]][0], "; sale aqui la ultima vez, y faltan mas"
    if mejor is None:
        cerca = difflib.get_close_matches(bloque[0], textos, n=1, cutoff=0.6)
        if not cerca:
            return 0, "; ni su primera linea, `%s`, ni otra parecida" % bloque[0]
        i = textos.index(cerca[0])
        return codigo[i][0], "; su primera linea ya no sale, y la mas parecida dice `%s`" % cerca[0]
    if mejor + casan >= len(textos):
        return codigo[-1][0], "; se acaba el fichero y falta `%s`" % bloque[casan]
    return codigo[mejor + casan][0], "; se aparta aqui: dice `%s` y el ancla `%s`" % (
        textos[mejor + casan], bloque[casan])


def _anclas_fork():
    """F.1 c): CODIGO_FORMULA contra el `_codigo` de security.rs, linea a linea; cuantas veces
    sale cada bloque de ANCLAS_CABLEADO en el `_codigo` de su fichero, y cuantas lineas de
    ANCLAS_LONGITUD son la regla sin su comentario de cola. Nada mas del fork. Devuelve los
    fallos."""
    fallos = []
    rel = os.path.relpath(FUENTE_FORMULA, RAIZ)
    try:
        hay = _codigo(_leer(FUENTE_FORMULA))
    except OSError as exc:
        fallos.append((rel, 0, "ILEGIBLE", str(exc)))
        hay = None
    if hay is not None:
        textos = [t for _, t in hay]
        dif = []
        for op, a1, a2, b1, b2 in difflib.SequenceMatcher(None, CODIGO_FORMULA, textos,
                                                           autojunk=False).get_opcodes():
            if op == "equal":
                continue
            for k in range(max(a2 - a1, b2 - b1)):
                esperada = CODIGO_FORMULA[a1 + k] if a1 + k < a2 else None
                dice = hay[b1 + k] if b1 + k < b2 else None
                n = dice[0] if dice else (hay[b1 - 1][0] if b1 else 0)
                if esperada is None:
                    dif.append((n, "sobra `%s`" % dice[1]))
                elif dice is None:
                    dif.append((n, "falta, tras esta linea, `%s`" % esperada))
                else:
                    dif.append((n, "dice `%s` y CODIGO_FORMULA `%s`" % (dice[1], esperada)))
        for n, que in dif[:8]:
            fallos.append((rel, n, "FORMULA", "%s: %s" % (que, REMEDIO_FORMULA)))
        if len(dif) > 8:
            fallos.append((rel, 0, "FORMULA", "y %d linea(s) mas que no casan" % (len(dif) - 8)))
    leidos = {}   # cada fichero se lee una vez: si no se puede, un ILEGIBLE y no uno por bloque
    for ruta, veces, funcion, bloque in ANCLAS_CABLEADO:
        rel = os.path.relpath(ruta, RAIZ)
        if ruta not in leidos:
            try:
                leidos[ruta] = _codigo(_leer(ruta))
            except OSError as exc:
                fallos.append((rel, 0, "ILEGIBLE", str(exc)))
                leidos[ruta] = None
        codigo = leidos[ruta]
        if codigo is None:
            continue
        textos = [t for _, t in codigo]
        n = len(bloque)
        sitios = [i for i in range(len(textos) - n + 1) if tuple(textos[i:i + n]) == bloque]
        if len(sitios) != veces:
            linea, donde = _donde(codigo, bloque, sitios, veces)
            fallos.append((rel, linea, "FORMULA", "`%s` (%d lineas) sale %d vez/veces, no %d%s: %s"
                           % (funcion, n, len(sitios), veces, donde, REMEDIO_FORMULA)))
    for ruta, linea, veces in ANCLAS_LONGITUD:
        try:
            hay = sum(t.split("//")[0].strip() == linea for t in _leer(ruta).split("\n"))
        except OSError as exc:
            fallos.append((os.path.relpath(ruta, RAIZ), 0, "ILEGIBLE", str(exc)))
            continue
        if hay != veces:
            fallos.append((os.path.relpath(ruta, RAIZ), 0, "LONGITUD",
                           "%d vez/veces, no %d: `%s`" % (hay, veces, linea)))
    return fallos


# ---- ATADO F: F.1, la replica de proof/security.rs con la aritmetica de f64 de Rust ----

def _div(a, b):
    if b == 0.0:
        return math.nan if (a == 0.0 or math.isnan(a)) else math.copysign(math.inf, a) * \
            math.copysign(1.0, b)
    return a / b


def _log2(x):
    if math.isnan(x) or x < 0:
        return math.nan
    if x == 0:
        return -math.inf
    return math.inf if math.isinf(x) else math.log2(x)


def _min_f64(xs):
    """`fold(f64::INFINITY, |a, b| a.min(b))`: `f64::min` descarta el NaN."""
    acc = math.inf
    for x in xs:
        acc = x if math.isnan(acc) else (acc if math.isnan(x) else min(acc, x))
    return acc


def _u64(x):
    """`as u64`: satura, y el NaN es 0."""
    if math.isnan(x) or x <= 0:
        return 0
    return 2 ** 64 - 1 if math.isinf(x) else int(x)


def _bits_consultas(o):
    """El termino de las consultas de la conjeturada: q * log2(blowup), mas la molienda si llega
    a 80."""
    c = (o["blowup"].bit_length() - 1) * o["q"]
    return c + o["molienda"] if c >= 80 else c


def _conjeturada(o):
    return min(min(BITS_CAMPO_BASE * o["ext"], _bits_consultas(o)) - 1, RESISTENCIA_HASH)


def _ldr_para_m(o, h, m):
    campo = float(BITS_CAMPO_BASE * o["ext"])
    m = float(m)
    rho = 1.0 / o["blowup"]
    alpha = (1.0 + 0.5 / m) * math.sqrt(rho)
    grado = o["blowup"] + 1.0
    lde = float(h * o["blowup"])
    h = float(h)
    lista = _div(m, rho - (2.0 * m / lde))
    eps = (
        -_log2(lista) - _log2(1.0) + campo,
        -_log2(lista * lista * (grado * (h + 2.0 - 1.0) + (h - 1.0))) + campo,
        campo - _log2((math.pow(m + 0.5, 7.0) / (3.0 * math.pow(rho, 1.5))) * math.pow(lde, 2.0)),
        o["molienda"] - _log2(math.pow(alpha, float(o["q"]))),
    )
    return _u64(_min_f64(eps))


def _ldr(o, h):
    r = (h + 2.0) / h
    m_max = min(_u64(math.ceil(1.0 / (2.0 * (math.sqrt(r) - 1.0)))), 1000)
    mejor = max(_ldr_para_m(o, h, m) for m in range(3, int(m_max)))
    return min(mejor, RESISTENCIA_HASH)


def _udr(o, h):
    campo = float(BITS_CAMPO_BASE * o["ext"])
    lde = h * o["blowup"]
    capas, d = 0, lde
    while d > (o["resto"] + 1) * o["blowup"]:   # FriOptions::num_fri_layers (winter-fri)
        d //= o["plegado"]
        capas += 1
    alpha = (1.0 + (h + 2.0) / float(lde)) * 0.5
    eps = [
        -_log2(1.0) + campo,
        -_log2((o["blowup"] + 1.0) * (h + 2.0 - 1.0) + (h - 1.0)) + campo,
        campo - _log2(float(lde)),
        _min_f64([campo - _log2((o["plegado"] - 1.0) * (lde + 1.0))] * capas),
        o["molienda"] - _log2(math.pow(alpha, float(o["q"]))),
    ]
    return min(_u64(_min_f64(eps)), RESISTENCIA_HASH)


def _deep(o, h):
    """El termino DEEP de la UDR, el tope de la conjeturada que su formula no descuenta (§708)."""
    return (-math.log2((o["blowup"] + 1.0) * (h + 2.0 - 1.0) + (h - 1.0))
            + BITS_CAMPO_BASE * o["ext"])


def _decimas(x):
    return int(math.floor(x * 10.0 + 0.5))


def derivacion():
    """F.0 y F.1. Devuelve (fallos, lo derivado o None si no hay de donde, cuentas)."""
    fallos = []
    try:
        src_capa = _leer(FUENTE_OPCIONES)
        src_test = _leer(FUENTE_TEST_CAPA)
        src_edad = _leer(FUENTE_EDAD)
        src_comp = _leer(FUENTE_COMPARACION)
    except OSError as exc:
        return [("crates/", 0, "ILEGIBLE", str(exc))], None, {}

    prod = _opciones_de(src_capa, "proof_options")
    edad = _opciones_de(src_edad, "opciones")
    if prod is None:
        return [(os.path.relpath(FUENTE_OPCIONES, RAIZ), 0, "OPCIONES",
                 "no se lee `proof_options()`: sin ella no hay de donde derivar")], None, {}
    if edad != prod:
        fallos.append((os.path.relpath(FUENTE_EDAD, RAIZ), 0, "OPCIONES",
                       "`opciones()` de la edad no son las de `proof_options()`: %s frente a %s"
                       % (edad, prod)))
    if prod["lotes"] != ("Linear", "Linear"):
        fallos.append((os.path.relpath(FUENTE_OPCIONES, RAIZ), 0, "OPCIONES",
                       "batching %s: la replica solo sabe el lineal, con el que la cifra no "
                       "depende del ancho ni de las restricciones" % (prod["lotes"],)))

    # F.1 c) - las anclas: CODIGO_FORMULA, las funciones de CABLEADO y las dos reglas de
    # longitud, en el fuente que las aplica.
    fallos.extend(_anclas_fork())

    # F.1 b) - la replica contra los KAT del fork, fuera del regimen de produccion.
    rel_formula = os.path.relpath(FUENTE_FORMULA, RAIZ)
    kat_casan = 0
    for linea, cifra, valor, q, blowup, molienda, ext, plegado, resto, log_h in KAT_FORK:
        o = {"q": q, "blowup": blowup, "molienda": molienda, "ext": ext, "plegado": plegado,
             "resto": resto}
        da = (_udr if cifra == "udr" else _ldr)(o, 2 ** log_h)
        kat_casan += da == valor
        if da != valor:
            fallos.append((rel_formula, linea, "KAT", "%s %d en el assert, y la replica da %d con "
                           "q = %d, blowup %d, molienda %d, extension de grado %d, plegado %d, "
                           "resto %d y 2^%d filas" % (cifra.upper(), valor, da, q, blowup,
                                                      molienda, ext, plegado, resto, log_h)))

    # Lo que fijan los tests.
    rel_test = os.path.relpath(FUENTE_TEST_CAPA, RAIZ)
    rel_edad = os.path.relpath(FUENTE_EDAD, RAIZ)
    fijo = {}
    for src, rel, nombre in ((src_test, rel_test, "CONJETURADA_MINIMA"), (src_test, rel_test,
                             "LDR_MINIMA"), (src_test, rel_test, "UDR_ACTUAL"),
                             (src_edad, rel_edad, "CONJETURADA_MINIMA"),
                             (src_edad, rel_edad, "UDR_ACTUAL")):
        v = _u32(src, nombre)
        if len(v) != 1:
            fallos.append((rel, 0, "TEST", "%s sale %d veces, se esperaba 1" % (nombre, len(v))))
            return fallos, None, {}
        if nombre in fijo and fijo[nombre] != v[0]:
            fallos.append((rel_edad, 0, "TEST", "%s = %d en la edad y %d en la capa: dos "
                           "productores de una cifra que nadie cruza"
                           % (nombre, v[0], fijo[nombre])))
        fijo.setdefault(nombre, v[0])
    n_piso, piso = _pares(src_edad, "PISO_LDR")
    n_deep, con_deep = _pares(src_edad, "CONJETURADA_CON_DEEP")
    sp = re.search(r"const SIN_PISO: \[u32; (\d+)\] = \[([^\]]*)\];", src_edad)
    sin_piso = [int(x) for x in re.findall(r"\d+", sp.group(2))] if sp else None
    ciclo = re.search(r"^pub const CICLO: usize = (\d+);", src_edad, re.M)
    if not piso or n_piso != len(piso) or not con_deep or n_deep != len(con_deep) \
            or sin_piso is None or not ciclo:
        fallos.append((rel_edad, 0, "TEST", "no se leen PISO_LDR, CONJETURADA_CON_DEEP, SIN_PISO "
                       "o CICLO tal como se declaran"))
        return fallos, None, {}
    ciclo = int(ciclo.group(1))
    if sorted(m for m, _ in piso) != sorted(m for m, _ in con_deep):
        fallos.append((rel_edad, 0, "TEST", "PISO_LDR y CONJETURADA_CON_DEEP no nombran las "
                       "mismas m"))

    # Las familias de longitud fija que prueba el test de la capa, con su TRACE_LENGTH.
    familias = []
    for modulo in dict.fromkeys(re.findall(r"\b(circuit_\w+)::TRACE_LENGTH", src_test)):
        try:
            familias.append((modulo, _trace_length(modulo)))
        except (OSError, ValueError) as exc:
            fallos.append((rel_test, 0, "FAMILIA", str(exc)))
    if len(familias) < 2:
        fallos.append((rel_test, 0, "FAMILIA", "el test prueba %d familias: sin la mas larga y la "
                       "mas corta no hay rango que atar" % len(familias)))
        return fallos, None, {}

    # F.1 a) - la replica contra lo que fijan los tests y lo que midio el §697.
    def nivel(o, filas):
        return {"conj": _conjeturada(o), "udr": _udr(o, filas), "ldr": _ldr(o, filas),
                "deep": _decimas(_deep(o, filas))}

    ldr_m, deep_m, udrs = {}, {}, set()
    pisos_casan = 0
    for m, valor in piso:
        n = nivel(edad or prod, 2 * (ciclo << m))
        ldr_m[m], deep_m[m] = n["ldr"], n["deep"]
        udrs.add(n["udr"])
        pisos_casan += n["ldr"] == valor
        if n["ldr"] != valor:
            fallos.append((rel_edad, 0, "REPLICA", "m = %d: la replica da LDR %d y PISO_LDR fija "
                           "%d; o el piso ya no es lo que da la funcion (el test solo exige >=, y "
                           "la igualdad es lo medido en el §697), o la replica se quedo atras"
                           % (m, n["ldr"], valor)))
        if n["udr"] != fijo["UDR_ACTUAL"]:
            fallos.append((rel_edad, 0, "REPLICA", "m = %d: UDR %d y la fijada es %d"
                           % (m, n["udr"], fijo["UDR_ACTUAL"])))
        if n["conj"] < fijo["CONJETURADA_MINIMA"]:
            fallos.append((rel_edad, 0, "REPLICA", "m = %d: conjeturada %d < %d"
                           % (m, n["conj"], fijo["CONJETURADA_MINIMA"])))
    for m, decimas in con_deep:
        if m in deep_m and deep_m[m] != decimas:
            fallos.append((rel_edad, 0, "REPLICA", "m = %d: la replica da DEEP %d decimas y "
                           "CONJETURADA_CON_DEEP fija %d" % (m, deep_m[m], decimas)))
    por_familia = []
    for modulo, t in familias:
        n = nivel(prod, 2 * t)
        por_familia.append((modulo, t, n))
        udrs.add(n["udr"])
        if n["conj"] < fijo["CONJETURADA_MINIMA"] or n["ldr"] < fijo["LDR_MINIMA"] \
                or n["udr"] != fijo["UDR_ACTUAL"]:
            fallos.append((rel_test, 0, "REPLICA", "%s (T = %d): conjeturada %d, LDR %d, UDR %d; "
                           "el test exige >= %d, >= %d y = %d"
                           % (modulo, t, n["conj"], n["ldr"], n["udr"],
                              fijo["CONJETURADA_MINIMA"], fijo["LDR_MINIMA"],
                              fijo["UDR_ACTUAL"])))
    larga = max(por_familia, key=lambda x: x[1])
    corta = min(por_familia, key=lambda x: x[1])

    # El circuito de comparacion: sus cuatro opciones, en el orden en que el test las recorre.
    rel_comp = os.path.relpath(FUENTE_COMPARACION, RAIZ)
    declaradas = {n: _args_opciones(a) for n, a in
                  re.findall(r"let (\w+) = ProofOptions::new\((.*?)\);", src_comp, re.S)}
    orden = re.search(r"for \(label, opts\) in \[(.*?)\]", src_comp, re.S)
    filas_comp = []
    try:
        t_comp = _trace_length(CIRCUITO_COMPARACION)
    except (OSError, ValueError) as exc:
        fallos.append((rel_comp, 0, "COMPARACION", str(exc)))
        t_comp = None
    pares = re.findall(r'\("([^"]+)",\s*(\w+)\)', orden.group(1)) if orden else ()
    for etiqueta, nombre in pares:
        o = declaradas.get(nombre)
        if o is None or t_comp is None:
            fallos.append((rel_comp, 0, "COMPARACION", "no se leen las opciones de %s" % nombre))
            continue
        fila = dict(o)
        fila.update(nivel(o, t_comp))
        fila["etiqueta"] = etiqueta
        filas_comp.append(fila)
    if len(filas_comp) != 4:
        fallos.append((rel_comp, 0, "COMPARACION", "%d configuraciones; FIVE_BACKENDS.md §4 "
                       "publica cuatro" % len(filas_comp)))

    d = {
        "opc": prod, "conj": _conjeturada(prod), "udr": udrs, "fijo": fijo,
        "ldr_todas": set(ldr_m.values()) | set(n["ldr"] for _, _, n in por_familia),
        "ldr_rango": tuple(sorted((larga[2]["ldr"], corta[2]["ldr"]))),
        "deep_rango": tuple(sorted((larga[2]["deep"], corta[2]["deep"]))),
        "ldr_m": ldr_m, "deep_m": deep_m, "sin_piso": sin_piso, "piso": dict(piso),
        "con_deep": dict(con_deep), "familias": por_familia,
        "bits_consultas": _bits_consultas(prod), "comparacion": filas_comp, "nivel": nivel,
    }
    cuentas = {"m": len(piso), "familias": len(familias), "comparacion": len(filas_comp),
               "pisos_casan": pisos_casan, "kat_casan": kat_casan}
    return fallos, d, cuentas


# ---- ATADO F: F.2, la prosa ----

def documentos_nivel():
    """El universo del ATADO F: el del C sin CITATION.cff, menos los excluidos aqui, mas los
    comentarios de FUENTES_RS_NIVEL."""
    for rel in documentos_url():
        if not rel.endswith(".md") or rel in EXCLUIDOS_NIVEL:
            continue
        if rel.startswith(tuple(PREFIJOS_EXCLUIDOS_NIVEL)):
            continue
        yield rel
    for rel in FUENTES_RS_NIVEL:
        yield rel


def _limpia(linea):
    """Quita el enfasis de Markdown, que parte una cifra de su etiqueta: «**59 en UDR**»."""
    linea = linea.replace(" ", " ").replace("**", "")
    return re.sub(r"(?<![\w`])_(?=\w)|(?<=\w)_(?![\w`])", "", linea)


def _unidades(rel):
    """(texto, [(desplazamiento, linea)], es_fila, abre_seccion) de un documento: los parrafos
    unidos, cada fila de tabla una unidad; de un .rs, solo los comentarios, y cada linea de codigo
    cierra el contexto."""
    lineas = _leer(os.path.join(RAIZ, rel)).split("\n")
    es_rs = rel.endswith(".rs")
    out, buf, mapa = [], [], []

    def cierra():
        if buf:
            out.append((" ".join(buf), list(mapa), False, False))
        del buf[:]
        del mapa[:]

    en_codigo = False
    for i, linea in enumerate(lineas, 1):
        if es_rs:
            m = re.match(r"^\s*//[/!]?\s?(.*)$", linea)
            if not m or not m.group(1).strip():
                cierra()
                if not m:
                    out.append(("", [], False, True))
                continue
            linea = m.group(1)
            if linea.lstrip().startswith("|"):
                cierra()
                out.append((_limpia(linea), [(0, i)], True, False))
                continue
        else:
            if linea.lstrip().startswith("```"):
                cierra()
                en_codigo = not en_codigo
                continue
            if en_codigo:
                continue
            if re.match(r"^#{1,6}\s", linea):
                cierra()
                out.append(("", [], False, True))
                continue
            if linea.lstrip().startswith("|"):
                cierra()
                out.append((_limpia(linea), [(0, i)], True, False))
                continue
            linea = re.sub(r"^\s*>\s?", "", linea)
            if not linea.strip():
                cierra()
                continue
        mapa.append((sum(len(b) + 1 for b in buf), i))
        buf.append(_limpia(linea))
    cierra()
    return out


CORTE_FRASE = re.compile(r"(?<=[.;])\s+(?=[«(¿¡\"⚠A-ZÁÉÍÓÚÑ])")


def _tramos(texto, es_fila):
    """Los (inicio, fin) de las frases de una unidad: los cortes de un parrafo, y en una fila los
    de cada celda, que una celda de ZENODO.md lleva dos frases y dos configuraciones."""
    def corta(a, b, patron):
        ini, out = a, []
        for m in patron.finditer(texto, a, b):
            out.append((ini, m.start()))
            ini = m.end()
        out.append((ini, b))
        return out
    celdas = corta(0, len(texto), re.compile(r"\|")) if es_fila else [(0, len(texto))]
    out = [t for a, b in celdas for t in corta(a, b, CORTE_FRASE)]
    return [(a, b) for a, b in out if texto[a:b].strip()]


# Una `m` o una longitud solo es la de la edad o la del DEEP si la frase lo dice: `m = 64` es el
# parametro de la ocultacion, y «la marca v1 con m = 64» no es un nivel (RFC-0009, RFC-0018).
CONTEXTO_M = re.compile(r"\bedad\b|\bage\b|\bLDR\b|\bDEEP\b")
# Toda clase de PATRONES_NIVEL lleva una de estas palabras, o la de CONTEXTO_M: un tramo sin
# ninguna no tiene cifra que leer, y no se le pasan los veintitantos patrones.
PALABRA_NIVEL = re.compile(r"UDR|LDR|DEEP|conjetur|conjectur|consultas|queries|blowup|molienda|"
                           r"grinding|demostrable|provable|extensi[oó]n|extension|ext\.|\bedad\b|"
                           r"\bage\b", re.I)


def _cifras(texto):
    """Las afirmaciones de nivel de un tramo: [(clase, valor, desplazamiento, como se lee)]."""
    tomado, out = [], []
    if not PALABRA_NIVEL.search(texto):
        return out
    for clase, patron in PATRONES_NIVEL:
        for m in patron.finditer(texto):
            if any(a < m.end() and m.start() < b for a, b in tomado):
                continue
            tomado.append((m.start(), m.end()))
            g = m.groups()
            if clase in ("ldr_m", "deep_m", "deep_L") and not CONTEXTO_M.search(texto):
                continue
            if clase == "deep_rango":
                if "DEEP" not in texto or re.match(r"\s*(?:KB|MB|MiB|ms|B\b)", texto[m.end():]):
                    continue
                v = tuple(sorted(_decimas(float(x.replace(",", "."))) for x in g))
            elif clase == "deep_m":
                v = (int(g[1]), _decimas(float(g[0].replace(",", "."))))
            elif clase == "deep_L":
                v = (int(g[1].replace(".", "")), _decimas(float(g[0].replace(",", "."))))
            elif clase in ("ldr_m",):
                v = (int(g[1]), int(g[0]))
            elif clase in ("par",):
                v = (int(g[0]), int(g[1]))
            elif clase == "ldr_rango":
                v = tuple(sorted((int(g[0]), int(g[1]))))
            elif clase == "molienda0":
                clase, v = "molienda", 0
            elif clase.startswith("ext"):
                clase, v = "ext", int(clase[3])
            else:
                v = int(g[0])
            out.append((clase, v, m.start(), m.group(0)))
    # La extension se ata junto a otra opcion: suelta, «la configuracion sin extension» describe
    # una configuracion, no la publica (PRINCIPIOS.md, SECURITY.md, la evaluacion de la
    # integracion).
    if not any(c in ("q", "blowup", "molienda") for c, _, _, _ in out):
        out = [x for x in out if x[0] != "ext"]
    return out


def _casa(d, conf, clase, v):
    """Si la configuracion `conf` -"P", la de produccion, o una fila de la comparacion- da la
    cifra."""
    if conf == "P":
        o = d["opc"]
        if clase == "conj":
            return v == d["conj"]
        if clase == "conj_min":
            return v == d["fijo"]["CONJETURADA_MINIMA"]
        if clase == "udr":
            return v in d["udr"] and v == d["fijo"]["UDR_ACTUAL"]
        if clase == "ldr":
            return v in d["ldr_todas"]
        if clase == "ldr_min":
            return v == d["fijo"]["LDR_MINIMA"]
        if clase == "ldr_rango":
            return v == d["ldr_rango"]
        if clase == "par":
            return v[0] in d["udr"] and v[1] in d["ldr_todas"]
        if clase == "deep_rango":
            return v == d["deep_rango"]
        if clase == "deep_L":
            return v[0] >= 8 and v[0] & (v[0] - 1) == 0 and _decimas(_deep(o, v[0])) == v[1]
        if clase == "deep_m":
            return d["deep_m"].get(v[0]) == v[1]
        if clase == "ldr_m":
            return d["ldr_m"].get(v[0]) == v[1]
        if clase == "bits_consultas":
            return v == d["bits_consultas"]
        return o.get(clase) == v
    if clase in ("conj", "udr", "ldr", "q", "blowup", "molienda", "ext"):
        return conf.get(clase) == v
    if clase == "par":
        return v == (conf["udr"], conf["ldr"])
    return False


def _dice(d, conf, clase):
    """Lo que la configuracion da para esa clase de cifra, para el mensaje del rojo."""
    if conf != "P":
        if clase == "par":
            return "%d-%d" % (conf["udr"], conf["ldr"])
        return str(conf.get(clase, "nada: esa cifra no es del circuito de comparacion"))
    tabla = {
        "conj": d["conj"], "conj_min": d["fijo"]["CONJETURADA_MINIMA"],
        "udr": d["fijo"]["UDR_ACTUAL"], "ldr": "una de %s" % sorted(d["ldr_todas"]),
        "ldr_min": d["fijo"]["LDR_MINIMA"], "ldr_rango": "%d-%d" % d["ldr_rango"],
        "deep_rango": "%.1f-%.1f" % tuple(x / 10.0 for x in d["deep_rango"]),
        "deep_L": "el DEEP de su longitud", "deep_m": "el DEEP de su m", "ldr_m": "la LDR de su m",
        "bits_consultas": d["bits_consultas"],
        "par": "UDR %d y una LDR de %s" % (d["fijo"]["UDR_ACTUAL"], sorted(d["ldr_todas"])),
    }
    return str(tabla.get(clase, d["opc"].get(clase)))


def _linea_de(mapa, desplazamiento):
    n = mapa[0][1] if mapa else 0
    for ini, linea in mapa:
        if ini <= desplazamiento:
            n = linea
    return n


def atado_f_prosa(d):
    """Devuelve (fallos, cifras, frases, saltadas, cifras de produccion por sitio, solas): solas
    cuenta las frases de una cifra sola que, sin marca propia, se leen como de produccion ("1"),
    las que llevan la marca de la comparacion y valen cualquier fila ("C"), y las de VARIAS cifras,
    todas de nivel, sin marca propia, que toman la fila heredada o cualquiera ("V")."""
    fallos, saltadas = [], []
    cifras = frases = 0
    solas = {"1": 0, "C": 0, "V": 0}
    por_sitio = {}
    usadas = set()
    filas = d["comparacion"]
    for rel in documentos_nivel():
        try:
            unidades = _unidades(rel)
        except OSError as exc:
            fallos.append((rel, 0, "ILEGIBLE", str(exc)))
            continue
        estado = None
        for texto, mapa, es_fila, abre in unidades:
            if abre:
                estado = None
                continue
            grupos = []
            # Las citas entre «»: otro texto, cualquier configuracion; y no marcan la frase.
            limpio = texto
            for m in re.finditer(r"«[^»]*»", texto):
                grupos.append((m.start(), m.group(0), "«»", None))
                limpio = limpio[:m.start()] + " " * (m.end() - m.start()) + limpio[m.end():]
            for a, b in _tramos(limpio, es_fila):
                tramo = limpio[a:b]
                p = bool(MARCA_PRODUCCION.search(tramo))
                c = bool(MARCA_COMPARACION.search(tramo))
                if p and not c:
                    clase, estado = "P", "P"
                elif c and not p:
                    clase, estado = "C", "C"
                elif p and c:
                    clase = "M"
                else:
                    clase = estado or "*"
                grupos.append((a, tramo, clase, c))
            for ini, tramo, clase, marca_c in sorted(grupos, key=lambda x: x[0]):
                hallado = _cifras(tramo)
                if not hallado:
                    continue
                ajena = [x for x in AJENAS_NIVEL if x[0] == rel and x[1] in tramo]
                if ajena:
                    usadas.update(ajena)
                    saltadas.append((rel, _linea_de(mapa, ini), "%s - %s" % (ajena[0][1],
                                                                              ajena[0][2])))
                    continue
                frases += 1
                cifras += len(hallado)
                # Una cifra de nivel SOLA -una conjeturada, una UDR, una LDR o un par- no elige
                # fila: sin la marca de la comparacion en su propia frase es la de produccion, haya
                # herencia o no. Con la marca, cualquier fila (y produccion, con las dos marcas).
                de_nivel = all(k in CLASES_DE_NIVEL for k, _, _, _ in hallado)
                if len(hallado) == 1 and de_nivel and clase != "«»":
                    if marca_c:
                        solas["C"] += 1
                    elif clase in ("*", "C"):
                        solas["1"] += 1
                        clase = "1"
                elif de_nivel and clase in ("*", "C") and not marca_c:
                    solas["V"] += 1   # varias, sin marca propia: la ceguera declarada
                candidatas = ["P"] if clase in ("P", "1") else filas if clase == "C" \
                    else ["P"] + filas
                # Una cifra que solo da produccion -un rango, una `m`, una longitud, un «≥»- deja
                # fuera las filas de la comparacion: no lo cambia el veredicto, que ninguna fila la
                # da, pero si contra que se dice el rojo.
                if "P" in candidatas and any(k in SOLO_PRODUCCION for k, _, _, _ in hallado):
                    candidatas = ["P"]
                if clase == "P":
                    # publica el nivel aunque lo publique mal: ese rojo es el suyo, no un SIN NIVEL
                    por_sitio[rel] = por_sitio.get(rel, 0) + len(hallado)
                if any(all(_casa(d, conf, k, v) for k, v, _, _ in hallado) for conf in candidatas):
                    continue
                # La candidata que menos falla dice que se esperaba; en empate, una frase que no es
                # de produccion se lee contra la comparacion.
                conf = min(candidatas, key=lambda cf: (sum(not _casa(d, cf, k, v)
                                                           for k, v, _, _ in hallado),
                                                       (cf == "P") != (clase == "P")))
                nombre = "produccion" if conf == "P" else "comparacion, «%s»" % conf["etiqueta"]
                for k, v, off, leido in hallado:
                    if not _casa(d, conf, k, v):
                        fallos.append((rel, _linea_de(mapa, ini + off), "NIVEL",
                                       "«%s»: %s da %s (frase %s)"
                                       % (leido.strip(), nombre, _dice(d, conf, k),
                                          {"P": "de produccion", "C": "de la comparacion",
                                           "M": "con las dos marcas", "«»": "entre «»",
                                           "*": "sin marca",
                                           "1": "de una cifra sola, sin la marca de la "
                                                "comparacion: es la de produccion"}[clase])))
    for ajena in AJENAS_NIVEL:
        if ajena not in usadas:
            fallos.append((ajena[0], 0, "AJENA", "el ancla «%s» ya no encuentra nada: una "
                           "exclusion muerta tapa la proxima" % ajena[1]))
    for rel in SITIOS_NIVEL:
        if rel not in por_sitio:
            fallos.append((rel, 0, "SIN NIVEL", "sitio conocido que ya no publica el nivel de "
                           "produccion: si es a proposito, sale de SITIOS_NIVEL con su razon"))
    return fallos, cifras, frases, saltadas, por_sitio, solas


def _celdas(linea):
    return [c.strip() for c in _limpia(linea).strip().strip("|").split("|")]


def _entero(texto):
    m = re.fullmatch(r"(\d{1,3}(?:\.\d{3})*|\d+)", texto.strip())
    return int(m.group(1).replace(".", "")) if m else None


def atado_f_tablas(d):
    """Las tres tablas, celda a celda. Devuelve (fallos, celdas atadas)."""
    fallos, celdas = [], 0
    o, nivel = d["opc"], d["nivel"]

    def filas_tras(lineas, i):
        out = []
        for j in range(i + 1, len(lineas)):
            if not lineas[j].lstrip().startswith("|"):
                break
            if re.match(r"^\|[\s|:-]+\|?\s*$", lineas[j]):
                continue
            out.append((j + 1, _celdas(lineas[j])))
        return out

    def lee(nombre):
        """Las lineas del documento, o None con su ILEGIBLE: una causa, un rojo."""
        try:
            return _leer(os.path.join(RAIZ, nombre)).split("\n")
        except OSError as exc:
            fallos.append((nombre, 0, "ILEGIBLE", str(exc)))
            return None

    # F.2.a - SECURITY.md §3.11, la tabla de longitud fija: filas = 2T, y su LDR y su DEEP.
    lineas = lee("SECURITY.md")
    cabezas = [i for i, l in enumerate(lineas or ()) if l.strip() == CABEZA_FIJAS]
    if lineas is None:
        pass
    elif len(cabezas) != 1:
        fallos.append(("SECURITY.md", 0, "TABLA", "la tabla de longitud fija sale %d veces, se "
                       "esperaba 1: «%s»" % (len(cabezas), CABEZA_FIJAS)))
    else:
        vistas = set()
        for n, c in filas_tras(lineas, cabezas[0]):
            fil = re.match(r"([\d.]+) filas$", c[0]) if c else None
            t = re.search(r"\(T = ([\d.]+)\)", c[3]) if len(c) > 3 else None
            if not fil or not t or len(c) != 4:
                fallos.append(("SECURITY.md", n, "TABLA", "fila que no se lee: %s"
                               % " | ".join(c)))
                continue
            h, t = _entero(fil.group(1)), _entero(t.group(1))
            vistas.add(h)
            r = nivel(o, h)
            celdas += 3
            if h != 2 * t:
                fallos.append(("SECURITY.md", n, "TABLA", "%d filas con T = %d: la traza oculta "
                               "tiene 2T" % (h, t)))
            if _entero(c[1]) != r["ldr"]:
                fallos.append(("SECURITY.md", n, "TABLA", "LDR %s con %d filas; se deriva %d"
                               % (c[1], h, r["ldr"])))
            if c[2] != ("%.1f" % (r["deep"] / 10.0)).replace(".", ","):
                fallos.append(("SECURITY.md", n, "TABLA", "DEEP %s con %d filas; se deriva %.1f"
                               % (c[2], h, r["deep"] / 10.0)))
        for modulo, t, _ in d["familias"]:
            if 2 * t not in vistas:
                fallos.append(("SECURITY.md", cabezas[0] + 1, "TABLA", "%s (T = %d), que el test "
                               "de la capa prueba, no tiene fila" % (modulo, t)))

    # F.2.b - SECURITY.md §3.11, las tablas de la edad: cada m, su piso y su DEEP; «—» solo sin
    # piso.
    vistas_m = []
    for i, l in enumerate(lineas or ()):
        if not l.startswith("| `m` |"):
            continue
        ms = [_entero(x.strip("`")) for x in _celdas(l)[1:]]
        resto = dict((c[0], (n, c[1:])) for n, c in filas_tras(lineas, i) if c)
        if "LDR" not in resto or "con el DEEP" not in resto:
            fallos.append(("SECURITY.md", i + 1, "TABLA", "una tabla de la edad sin su fila LDR o "
                           "«con el DEEP»"))
            continue
        for j, m in enumerate(ms):
            vistas_m.append(m)
            for fila, de_test, deriv in (("LDR", d["piso"], d["ldr_m"]),
                                         ("con el DEEP", d["con_deep"], d["deep_m"])):
                n, valores = resto[fila]
                celda = valores[j] if j < len(valores) else ""
                celdas += 1
                if m in d["sin_piso"]:
                    if celda != "—":
                        fallos.append(("SECURITY.md", n, "TABLA", "m = %d esta en SIN_PISO y la "
                                       "tabla le da %s" % (m, celda)))
                    continue
                if m not in de_test:
                    fallos.append(("SECURITY.md", n, "TABLA", "m = %d no tiene cifra en el test"
                                   % m))
                    continue
                if fila == "LDR":
                    esperada = str(deriv[m])
                else:
                    esperada = ("%.1f" % (deriv[m] / 10.0)).replace(".", ",")
                if celda != esperada:
                    fallos.append(("SECURITY.md", n, "TABLA", "%s de m = %d: dice %s y se "
                                   "deriva %s" % (fila, m, celda, esperada)))
    declaradas = sorted(set(d["piso"]) | set(d["sin_piso"]))
    if lineas is not None and sorted(vistas_m) != declaradas:
        fallos.append(("SECURITY.md", 0, "TABLA", "las tablas de la edad cubren m = %s y el test "
                       "declara %s" % (sorted(vistas_m), declaradas)))

    # F.2.c - FIVE_BACKENDS.md §4: una fila por configuracion del test, en su orden.
    lineas = lee("FIVE_BACKENDS.md")
    if lineas is None:
        return fallos, celdas
    cabezas = [i for i, l in enumerate(lineas) if l.strip() == CABEZA_COMPARACION]
    if len(cabezas) != 1:
        fallos.append(("FIVE_BACKENDS.md", 0, "TABLA", "la tabla de la comparacion sale %d veces"
                       % len(cabezas)))
        return fallos, celdas
    filas = filas_tras(lineas, cabezas[0])
    if len(filas) != len(d["comparacion"]):
        fallos.append(("FIVE_BACKENDS.md", cabezas[0] + 1, "TABLA", "%d filas y el test recorre "
                       "%d configuraciones" % (len(filas), len(d["comparacion"]))))
    for (n, c), conf in zip(filas, d["comparacion"]):
        if len(c) != 5:
            fallos.append(("FIVE_BACKENDS.md", n, "TABLA", "fila que no se lee"))
            continue
        celdas += 3
        for k, v, _, leido in _cifras(c[0]):
            if conf.get(k) != v:
                fallos.append(("FIVE_BACKENDS.md", n, "TABLA", "«%s», y la configuracion «%s» del "
                               "test tiene %s" % (leido, conf["etiqueta"], conf.get(k))))
        if c[3] != "%d bits" % conf["conj"]:
            fallos.append(("FIVE_BACKENDS.md", n, "TABLA", "conjeturada «%s»; se deriva %d"
                           % (c[3], conf["conj"])))
        if re.sub(r"\s", "", c[4]) != "%d/%d" % (conf["udr"], conf["ldr"]):
            fallos.append(("FIVE_BACKENDS.md", n, "TABLA", "demostrable «%s»; se deriva %d / %d"
                           % (c[4], conf["udr"], conf["ldr"])))
    return fallos, celdas


def atado_f():
    """Devuelve (fallos, resumen para imprimir)."""
    fallos, d, cuentas = derivacion()
    if d is None:
        return fallos, ["ATADO F: sin derivacion; ver los rojos"], 0
    fallos_p, cifras, frases, saltadas, por_sitio, solas = atado_f_prosa(d)
    fallos_t, celdas = atado_f_tablas(d)
    o = d["opc"]
    # «casa» solo si la derivacion no tiene ningun rojo -REPLICA, KAT, FORMULA, ILEGIBLE...-; si
    # los tiene, cuantos casan.
    if fallos:
        casa = ("y con los pisos de la edad casa en %d de %d, con los KAT del fork en %d de %d, y "
                "la derivacion tiene %d rojo(s) (%s: abajo)"
                % (cuentas["pisos_casan"], cuentas["m"], cuentas["kat_casan"], len(KAT_FORK),
                   len(fallos), ", ".join(sorted(set(f[2] for f in fallos)))))
    else:
        casa = ("y casa con los %d pisos de la edad y con los %d KAT del fork"
                % (cuentas["m"], len(KAT_FORK)))
    resumen = [
        "ATADO F: opciones %d/%d/%d, extension de grado %d; la replica de security.rs da "
        "conjeturada %d, UDR %s, LDR %d-%d y DEEP %.1f-%.1f en las %d familias del test, %s; %d "
        "configuraciones de la comparacion"
        % (o["q"], o["blowup"], o["molienda"], o["ext"], d["conj"],
           "/".join(str(x) for x in sorted(d["udr"])), d["ldr_rango"][0], d["ldr_rango"][1],
           d["deep_rango"][0] / 10.0, d["deep_rango"][1] / 10.0, cuentas["familias"], casa,
           cuentas["comparacion"]),
        "ATADO F: anclas en el codigo del fork, y nada mas de ese codigo: las %d lineas de "
        "security.rs hasta `mod tests` (lo de detras no se lee) y %d funcion(es) del cableado, "
        "entera(s), %d lineas, cada una las veces que dice CABLEADO; cualquier otra linea de ese "
        "codigo, tambien una por la que pase una entrada de la formula, no se vigila aqui "
        "(ceguera de la cabecera)"
        % (len(CODIGO_FORMULA), len(ANCLAS_CABLEADO),
           sum(len(b) for _, _, _, b in ANCLAS_CABLEADO)),
        "ATADO F: %d cifras de nivel en %d frases; de produccion en %d sitio(s); %d celdas de "
        "tablas" % (cifras, frases, len(por_sitio), celdas),
        "ATADO F: %d frase(s) de una cifra sola, sin marca propia, leidas como de produccion; %d "
        "con la marca de la comparacion, que valen cualquier fila, y %d de varias cifras de nivel "
        "sin marca propia, que toman la heredada o cualquiera (ceguera declarada)"
        % (solas["1"], solas["C"], solas["V"]),
    ]
    heredados = sorted(HEREDADOS_NIVEL.items())
    for rel, razon in sorted(EXCLUIDOS_NIVEL.items()) + sorted(PREFIJOS_EXCLUIDOS_NIVEL.items()) \
            + heredados:
        resumen.append("excluido (nivel) %s - %s" % (rel, razon))
    for rel, n, detalle in saltadas:
        resumen.append("SALTADA  %-18s :%-5d %s" % (rel, n, detalle))
    fallos = fallos + fallos_p + fallos_t
    del_c = set(EXCLUIDOS_URL) | set(PREFIJOS_EXCLUIDOS_URL)
    if set(HEREDADOS_NIVEL) != del_c:
        fallos.append(("(universo)", 0, "EXCLUSION", "el ATADO C excluye %s y HEREDADOS_NIVEL da "
                       "razon de %s: una exclusion heredada sin razon aqui no se imprime"
                       % (sorted(del_c), sorted(HEREDADOS_NIVEL))))
    if cifras < MINIMO_CIFRAS_F:
        fallos.append(("(universo)", 0, "VIDA", "%d cifras de nivel, y el minimo medido es %d: un "
                       "censo que encoge se declara" % (cifras, MINIMO_CIFRAS_F)))
    if celdas < MINIMO_CELDAS_F:
        fallos.append(("(tablas)", 0, "VIDA", "%d celdas, y el minimo medido es %d"
                       % (celdas, MINIMO_CELDAS_F)))
    return fallos, resumen, cifras


def main():
    pago_b, mib, msi, fecha = constante()
    pares_mib = {tuple(x.split(".")) for x in mib}
    pares_msi = {tuple(x.split(".")) for x in msi}

    fallos = []
    saltadas = []
    vistos = 0

    for nombre in documentos():
        ruta = os.path.join(RAIZ, nombre)
        with open(ruta, encoding="utf-8") as fh:
            lineas = fh.readlines()
        for i, linea in enumerate(lineas):
            ini = max(0, i - VENTANA)
            fin = min(len(lineas), i + VENTANA + 1)
            bloque = "".join(lineas[ini:fin])
            if not CONTEXTO.search(bloque):
                continue
            for m in CIFRA.finditer(linea):
                par = (m.group(1), m.group(2))
                unidad = m.group(3)
                citada = par[0] if par[1] is None else "%s,%s" % par
                if HISTORICA.search(bloque):
                    saltadas.append(
                        (nombre, i + 1,
                         "%s %s - cita de una version anterior"
                         % (citada, unidad))
                    )
                    continue
                vistos += 1
                if par in pares_mib and unidad == "MiB":
                    continue
                if par in pares_msi and unidad == "MB":
                    continue
                if par in pares_mib and unidad == "MB":
                    fallos.append(
                        (nombre, i + 1, "UNIDAD",
                         "el valor binario lleva etiqueta MB; o MiB, o el "
                         "equivalente SI %s-%s MB" % msi)
                    )
                else:
                    fallos.append(
                        (nombre, i + 1, "VALOR",
                         "dice %s %s; se esperan %s o %s MiB, o %s o %s MB"
                         % ((citada, unidad) + mib + msi))
                    )

    print("check_publicadas: %d..%d B por pago -> %s-%s MiB / %s-%s MB por mil (banda "
          "medida el %s)" % (pago_b + mib + msi + (fecha,)))
    print("  %d citas examinadas en documentos del repo" % vistos)
    for nombre, razon in sorted(EXCLUIDOS.items()):
        print("  excluido %s - %s" % (nombre, razon))
    print("  excluido doc/preprints/ - entrada 28, SUSPENDIDA hasta el fin del proyecto")
    for nombre, n, detalle in saltadas:
        print("  SALTADA  %-18s :%-5d %s" % (nombre, n, detalle))

    fallos_c, nuevas, recorridos = atado_c()
    print("  ATADO C: %d documentos recorridos; la URL del repositorio citada %d vez/veces"
          % (recorridos, nuevas))
    for nombre, razon in sorted(EXCLUIDOS_URL.items()):
        print("  excluido (URL) %s - %s" % (nombre, razon))
    print("  excluido (URL) doc/preprints/ - entrada 28; lo que alli quede se DECLARA")

    fallos_d, n_rfc, n_sitios, n_cuentas = atado_d()
    print("  ATADO D: %d RFC con estado propio; %d cuenta(s) en %d sitio(s), y la fila de "
          "cada uno en %s" % (n_rfc, n_cuentas, n_sitios, FILA_RFC))

    fallos_e, n_vec, citas_e = atado_e()
    print("  ATADO E: spec/vectors/ tiene %d ficheros; %d cita(s) de esa cuenta en %s"
          % (n_vec, citas_e, " y ".join(SITIOS_VECTORES)))

    fallos_g, n_g, completos_g, citas_g, plantilla_g = atado_g()
    print("  ATADO G: %d RFC %s; %d con su autor y su vineta de GenAI, sin "
          "identificador de modelo, y %d por excepcion"
          % (n_g, "y la plantilla" if plantilla_g else "y falta la plantilla", completos_g,
             len(EXCEPCIONES_G)))
    for num, asiento, linea in citas_g:
        print("  excepcion (G) RFC-%s - «%s», sin vineta de GenAI: AUDITORIA.md:%d, en el %s"
              % (num, EXCEPCIONES_G[num][2], linea, asiento))

    fallos_f, resumen_f, cifras_f = atado_f()
    for linea in resumen_f:
        print("  " + linea)

    fallos_bf, filas_b = atado_b_filas(pago_b)
    print("  ATADO B, las filas: %d de %d filas de tabla llevan su banda de metrics.rs"
          % (filas_b - len([f for f in fallos_bf if f[2] == "BANDA"]), len(FILAS_BANDA)))

    fallos_r, marcadas_r, excepcion_r, recorridos_r = retiradas()
    print("  RETIRADAS: %d fichero(s); %d cita(s) de %s tras su marca y %d por excepcion"
          % (recorridos_r, marcadas_r, ", ".join("«%s»" % e for e, _ in RETIRADAS), excepcion_r))
    for nombre, razon in sorted(EXCLUIDOS_RETIRADAS.items()):
        print("  excluido (RETIRADAS) %s - %s" % (nombre, razon))
    for nombre, razon in sorted(PREFIJOS_EXCLUIDOS_RETIRADAS.items()):
        print("  excluido (RETIRADAS) %s - %s" % (nombre, razon))
    for nombre, _, razon in CITAS_RETIRADAS:
        print("  excepcion (RETIRADAS) %s - %s" % (nombre, razon))

    if vistos == 0:
        print("ROJO: CERO citas encontradas.")
        print("      Un censo vacio no es un hallazgo: es un instrumento que")
        print("      dejo de ver. Revisa el patron antes de creerte el cero.")
        return 1

    if fallos:
        print("")
        print("ROJO: %d citas no cuadran con la fuente unica" % len(fallos))
        for nombre, n, clase, detalle in fallos:
            print("  %-18s :%-5d %-7s %s" % (nombre, n, clase, detalle))
        print("")
        print("  La constante vive en crates/zk-ssl/src/metrics.rs.")
        print("  Si el rojo es del SISTEMA y no del documento, quien habla es")
        print("  el otro eslabon: la_cifra_publicada_sigue_siendo_la_medida.")
        return 1

    if fallos_c:
        print("")
        print("ROJO: %d sitio(s) no cuadran con la URL del repositorio" % len(fallos_c))
        for nombre, n, clase, detalle in fallos_c:
            print("  %-40s :%-5d %-19s %s" % (nombre, n, clase, detalle))
        print("")
        print("  La URL vive en URL_REPO, aqui arriba, y en CITATION.cff.")
        return 1

    if fallos_d:
        print("")
        print("ROJO: %d sitio(s) no dicen del RFC lo que el RFC dice de si mismo"
              % len(fallos_d))
        for nombre, n, clase, detalle in fallos_d:
            print("  %-40s :%-5d %-10s %s" % (nombre, n, clase, detalle))
        print("")
        print("  El estado lo declara la linea `- **Estado:**` de cada spec/rfc/NNNN-*.md,")
        print("  y el vocabulario, spec/rfc/PROCESO.md. Si el rojo es del RFC y no del")
        print("  documento, quien se quedo atras es el RFC.")
        return 1

    if fallos_e:
        print("")
        print("ROJO: %d sitio(s) no dicen de spec/vectors/ lo que el arbol cuenta" % len(fallos_e))
        for nombre, n, clase, detalle in fallos_e:
            print("  %-40s :%-5d %-10s %s" % (nombre, n, clase, detalle))
        print("")
        print("  La cuenta es la del arbol: todo fichero bajo spec/vectors/, recursivo.")
        return 1

    if fallos_g:
        print("")
        print("ROJO: %d fallo(s) del ATADO G (§846): el autor y la asistencia GenAI de los RFC"
              % len(fallos_g))
        for nombre, n, clase, detalle in fallos_g:
            print("  %-40s :%-5d %-9s %s" % (nombre, n, clase, detalle))
        print("")
        print("  Cada spec/rfc/NNNN-*.md lleva antes de su primer `## ` una «- **Autor:**» con")
        print("  un nombre de GENAI.md y una «- **Asistencia GenAI:**» sin identificador de")
        print("  modelo. Una excepcion va a EXCEPCIONES_G con la frase de su asiento.")
        return 1

    if fallos_f:
        print("")
        print("ROJO: %d cifra(s) o constante(s) de nivel no cuadran con lo que se deriva"
              % len(fallos_f))
        for nombre, n, clase, detalle in fallos_f:
            print("  %-40s :%-5d %-11s %s" % (nombre, n, clase, detalle))
        print("")
        print("  Las cifras se derivan de proof_options() (crates/zk-ssl/src/lib.rs) con la")
        print("  replica de crates/winter-air/src/proof/security.rs, contrastada con los tests")
        print("  del §697 y el §708 y con los KAT del fork. Un rojo REPLICA, KAT, FORMULA o TEST")
        print("  es del codigo; uno NIVEL o TABLA, del documento. Un FORMULA tras un rustfmt")
        print("  o una migracion del fork que no cambia la formula se arregla actualizando")
        print("  CODIGO_FORMULA o CABLEADO en el MISMO sello que toco el fork; que la")
        print("  replica siga siendo la funcion lo dicen los tests de Rust del canon")
        print("  (tools/canon.sh:122-123), no los verdes REPLICA y KAT de aqui.")
        return 1

    if fallos_bf:
        print("")
        print("ROJO: %d fila(s) de tabla no dan la banda de metrics.rs (ATADO B, §841)"
              % len(fallos_bf))
        for nombre, n, clase, detalle in fallos_bf:
            print("  %-18s :%-5d %-7s %s" % (nombre, n, clase, detalle))
        print("")
        print("  Las bandas viven en crates/zk-ssl/src/metrics.rs: BANDA_ENVIO_B y BANDA_COBRO_B")
        print("  en `mod tests`, PUBLICADA_PAGO_MIN_B y PUBLICADA_PAGO_MAX_B arriba. Quien mueve")
        print("  una mueve las filas de FILAS_BANDA en el mismo sello.")
        return 1

    if fallos_r:
        print("")
        print("ROJO: %d cifra(s) retirada(s) publicada(s) como de hoy (§841)" % len(fallos_r))
        for nombre, n, clase, detalle in fallos_r:
            print("  %-40s :%-5d %-9s %s" % (nombre, n, clase, detalle))
        print("")
        print("  Una cifra de RETIRADAS solo queda como cita de lo que se decia: con «decía»,")
        print("  «it said» o ⚠️ delante, en su linea o su frase. Si la frase es legitima sin")
        print("  marca, va a CITAS_RETIRADAS con su ancla y su razon.")
        return 1

    if marcadas_r < MINIMO_RETIRADAS:
        print("ROJO: RETIRADAS ve %d cita(s) marcada(s), menos que su suelo, %d."
              % (marcadas_r, MINIMO_RETIRADAS))
        print("      Un censo que encoge sin asiento es un instrumento que dejo de ver: revisa")
        print("      el universo y los patrones antes de bajar el suelo.")
        return 1

    if cifras_f == 0:
        print("ROJO: el ATADO F no encuentra ni una cifra de nivel.")
        print("      Un censo vacio no es un hallazgo: revisa PATRONES_NIVEL y el universo.")
        return 1

    if nuevas == 0:
        print("ROJO: la URL del repositorio no se cita ni una vez.")
        print("      Un censo vacio no es un hallazgo: revisa URL_REPO y el universo.")
        return 1


    print("OK: todas las citas cuadran en valor y en unidad, y la URL del repositorio es una")
    return 0


if __name__ == "__main__":
    sys.exit(main())
