# Evaluación: una firma poscuántica de menos de 100 bytes, sin estado

**La pregunta, tal como se hizo** (30-09-2026): con lo que la casa maneja —la firma de cabezas
basada en hashes y con estado, el guardián de su índice, los árboles de Merkle del acuse y del
consumo, y las pruebas STARK de la vía de pago—, ¿se puede diseñar una firma resistente a
ordenadores cuánticos **de menos de 100 bytes**, «frente a los 2.400 de Dilithium», **sin perder
seguridad y sin árboles de estado**?

**Respuesta corta: no.** Las tres condiciones juntas chocan con una aritmética que este árbol ya
tiene escrita en un test, y todo lo que hoy baja de 100 bytes descansa en una familia de supuestos
que la casa rechazó en `AUDITORIA.md` §106 (con la excepción del §705 para el medio del ancla,
ML-DSA-44, que es de retículos y no de esa familia). Lo que sí sale de las piezas de la casa es
otra cosa, más pequeña y medible, y se dice por su nombre en §6.

**Estado**: evaluada, sin código. Medido sobre `da6a768` (S588) en la máquina de la sesión.

**Procedencia**: escrita en una sesión de trabajo con asistencia de IA generativa, en una rama que no
se fusionó, e integrada a nombre del autor en el §597 de `AUDITORIA.md`. Integrar no es aceptar: la
aceptación que describe [`GENAI.md`](../GENAI.md) queda pendiente.

**Convención**: ✅ medido en este árbol, con el comando al lado · 📐 derivado con una aritmética
que se reproduce con el script de abajo · 📖 leído en la fuente primaria, con cita · ⚠️ matiz ·
**«espejo»** = el original no era alcanzable desde la red de esta sesión (el proxy devolvió 403 a
nist.gov, ietf.org, rfc-editor.org, eprint.iacr.org y arxiv.org) y se leyó una copia **idéntica en
dos espejos independientes**; cotejar con el original es un pendiente declarado (§8).

---

## 1. Lo que la casa tiene, medido en este árbol ✅

Los 341 vectores JSON de `spec/vectors/` son lo que la casa publica y lo que un tercero verifica
con el nodo apagado. Pesado pieza a pieza, en **bytes binarios** (en el cable van en hex, el doble):

| pieza | bytes | dónde se mide | ocurrencias |
|---|---|---|---|
| firma XMSS de la cabeza de época | **18.519** | `FIRMA_RFC_BYTES` (18.469, la cifra del RFC 8391 para `XMSSMT-SHA2_40/8_256`) + 50 del preámbulo `ZK-SSL-epoch-head` ‖ versión ‖ digest | 314 |
| cofirma de un testigo | **18.593** | la misma firma + 124 del preámbulo de cofirma | 11 |
| camino de acuse o de recibo, 32 niveles | **1.024** | `RECEP_DEPTH` = 32 en `zk-ssl-verify`; 32 hermanos × 32 B | 74 vectores |
| camino del árbol de consumos, 63 niveles | **2.016** | 63 hermanos × 32 B | 45 vectores |
| prueba STARK de una transición | **43.092 a 78.451** | por vector; la banda por pago, dos pruebas, es `PUBLICADA_PAGO_MIN_B..MAX_B` en `metrics.rs` | 99 |

Reproducir, desde la raíz del repositorio:

```python
# Mide, sobre spec/vectors/**/*.json, lo que pesa cada pieza de evidencia que la casa publica.
import json, glob, collections
alturas, firmas, cofirmas, pruebas, n = collections.Counter(), collections.Counter(), collections.Counter(), [], 0
def rec(o, ruta=""):
    if isinstance(o, dict):
        if isinstance(o.get("siblings"), list): alturas[len(o["siblings"])] += 1
        for k, v in o.items():
            if k == "signature" and isinstance(v, str): firmas[(len(v) - 2) // 2] += 1
            if k == "firma" and isinstance(v, str) and "cofirmas" in ruta: cofirmas[(len(v) - 2) // 2] += 1
            if k in ("prueba", "proof") and isinstance(v, str) and len(v) > 1000: pruebas.append((len(v) - 2) // 2)
            rec(v, ruta + "/" + k)
    elif isinstance(o, list):
        for v in o: rec(v, ruta)
for f in sorted(glob.glob("spec/vectors/**/*.json", recursive=True)):
    try: d = json.load(open(f))
    except ValueError: continue
    n += 1; rec(d)
print("vectores JSON leidos:", n)
print("camino: niveles -> vectores:", dict(sorted(alturas.items())))
print("firma de cabeza (bytes -> ocurrencias):", dict(firmas))
print("cofirma de testigo (bytes -> ocurrencias):", dict(cofirmas))
print("pruebas STARK: n =", len(pruebas), "min", min(pruebas), "max", max(pruebas))
```

Salida el 30-09-2026: `{2: 1, 3: 1, 31: 3, 32: 74, 62: 1, 63: 45}` · `{18519: 314}` · `{18593: 11}`
· `n = 99 min 43092 max 78451`.

⚠️ **Re-medido al integrar (§597), con este mismo script sobre `be8aa31`:** 364 vectores leídos; la
firma de cabeza, 18.519 B en 340 ocurrencias —el ancla del §593 añadió 26—; los caminos, la cofirma
y las pruebas STARK, idénticos. Los tamaños no cambian; sólo el recuento, que es de su fecha.

**Lo que enseña la tabla antes de mirar fuera.** La casa ya tiene **dos** firmas de la familia del
hash, y las dos están a cientos de veces de los 100 bytes:

- **Con estado**: XMSS^MT, 18.469 B por cabeza. El «árbol de estado» que la pregunta quiere evitar
  es exactamente el que la casa paga con el guardián del índice (§234) para que una firma sea
  multiuso.
- **Sin estado**: la autoridad de gasto no es una firma clásica, es **conocimiento de preimagen
  probado en STARK** (`SECURITY.md` §3, `AUDITORIA.md` §117). Es una firma en sentido estricto,
  solo hash, sin índice que gastar, y pesa entre 43 y 78 KB por transición.

Y la fórmula que las une ya está en un test: `el_ancho_del_indice_sale_del_conjunto_de_xmss`, en
`crates/zk-ssl-verify/src/lib.rs`, ata `SIG_LEN` a `índice + n + d·(2n+3)·n + h·n` con las fórmulas
del RFC 8391. Ese `(2n+3)·n` = 67 · 32 = 2.144 B 📐 es una sola firma de un solo uso, **antes** de
ningún árbol (el RFC escribe `len = 67`, Tabla 1; el producto es derivado).

---

## 2. La aritmética que cierra la vía del hash 📐

Toda firma basada en hashes revela cadenas de preimágenes. En Winternitz (WOTS+, RFC 8391 §3.1.1),
con elementos de `n` bytes, resumen de `ℓ` bits y parámetro `w`:

- `len_1 = ⌈ℓ / log2 w⌉` cadenas para el mensaje;
- `len_2 = ⌊log_w(len_1 · (w − 1))⌋ + 1` cadenas para el checksum;
- firma = `(len_1 + len_2) · n` bytes; verificar cuesta, en el peor caso, `(len_1 + len_2) · (w − 1)`
  llamadas al hash.

Con `n = 32` y `ℓ = 256`, en enteros exactos:

| `log2 w` | `len_1` | `len_2` | `len` | firma OTS | hashes por verificación, peor caso |
|---|---|---|---|---|---|
| 4 (el RFC) | 64 | 3 | **67** | **2.144 B** | 1.005 |
| 8 | 32 | 2 | 34 | 1.088 B | 8.670 |
| 16 | 16 | 2 | 18 | 576 B | ≈ 1,2 · 10⁶ |
| 32 | 8 | 2 | 10 | 320 B | ≈ 4,3 · 10¹⁰ |
| 64 | 4 | 2 | 6 | 192 B | ≈ 1,1 · 10²⁰ |

La fila del RFC da `len = 67`, que es la Tabla 1 de RFC 8391 y el `2n+3` del test de §1: la fórmula
está bien.

**El umbral de los 100 bytes.** Con elementos de 32 bytes caben **3**. Con la fórmula del RFC
`len_2 ≥ 2` en cuanto `len_1 ≥ 2`, así que `len` **nunca vale 3**: el primer `w` con `len ≤ 3` es
`log2 w = 256` (`len_1 = 1`, `len = 2`, 64 B). Sin checksum, una variante que el RFC no tiene,
`len_1 ≤ 3` exige `log2 w ≥ 86`. En cualquiera de los dos casos verificar cuesta **más de 2⁸⁵
llamadas al hash por cadena**. No es un límite de ingenio: es que cada elemento de 32 bytes solo
puede comprimir `log2 w` bits del resumen, y `w` es el coste de recorrerlo.

Y esto es **solo la firma de un solo uso**. Hacerla multiuso añade, o bien el camino de
autenticación y el índice (XMSS: `h · n` más, y el estado), o bien el hiperárbol y FORS (SLH-DSA:
7.856 B en su conjunto más corto, §3).

Las salidas que no valen, y por qué:

- **Resumen más corto o elementos de 16 bytes.** Con Grover, 128 bits de resistencia poscuántica a
  segunda preimagen exigen 256 bits de salida; bajar a 16 bytes deja 64. Viola «sin perder
  seguridad», y la casa ya midió lo que cuesta un techo de 63 bits (`SECURITY.md` §3).
- **Comprimir la firma con una prueba sucinta.** Es lo que la casa hace con la autoridad de gasto,
  y pesa 43 a 78 KB (§1). Las pruebas de 128 a 192 bytes son Groth16 sobre curvas con
  emparejamiento (192 B en BLS12-381, especificación de Zcash), y la casa las descartó por eso
  (`FIVE_BACKENDS.md`). Lo poscuántico, leído en fuente (conversiones de los PDF de ePrint, espejo):
  LaBRADOR, 58 KB para un R1CS de 2²⁰ restricciones a 128 bits (retículos); Greyhound, 53 KB
  (resumen); WHIR-CB, 157 KiB, frente a 306 KiB de FRI en la misma configuración (hash); Aurora,
  de 40 a 130 kB (hash). **El más pequeño de 2025-2026 encontrado**, LUNA+, 4,22 KB (por resumen, sin lectura
  primaria), es de verificador designado y con un CRS de 0,54 GB. Ninguno implementado baja de 1 KB; el único
  resultado del orden de λ bits es teórico, de verificador designado y bajo un supuesto no
  estándar (evasive LWE). Ishai, Su y Wu lo cuantificaron en 2021: «a 1000× gap in the proof size
  between the best pre-quantum constructions and the best post-quantum ones».

⚠️ La cota `≥ 2^(256/k)` para `k` elementos vale para verificadores que recorren cadenas o árboles
de hash (Winternitz, Lamport, los de grafo); no es un teorema para toda función de verificación
concebible. Se declara como lo que es: la aritmética de las construcciones conocidas.

Reproducir:

```python
n, ell = 32, 256
def fila(lw):
    w = 1 << lw
    len1 = -(-ell // lw)                       # ceil(ell / log2 w)
    x, len2 = len1 * (w - 1), 0
    while x: x //= w; len2 += 1                # floor(log_w(len1 * (w - 1))) + 1
    ln = len1 + len2
    return lw, len1, len2, ln, ln * n, ln * (w - 1)
for lw in (4, 8, 16, 32, 64): print(fila(lw))
```

---

## 3. Dónde existen hoy firmas de ese tamaño 📖

Cifras de nivel 1 de NIST salvo donde se dice otra cosa, en bytes, a 30-09-2026. La columna
«fuente» dice de dónde se leyó cada cifra; «espejo» según la convención de arriba.

| esquema · conjunto | categoría NIST | firma | clave pública | supuesto | estado | fuente |
|---|---|---|---|---|---|---|
| ML-DSA-44 | **2** | 2.420 | 1.312 | retículos | FIPS 204, 13-08-2024 | FIPS 204, Tabla 2 (espejo) |
| ML-DSA-65 | 3 | 3.309 | 1.952 | retículos | FIPS 204 | idem |
| SLH-DSA-SHA2-128s | 1 | **7.856** | 32 | solo hash, sin estado | FIPS 205, 13-08-2024 | FIPS 205, Tabla 2 (espejo) |
| SLH-DSA-SHA2-128f | 1 | 17.088 | 32 | solo hash, sin estado | FIPS 205 | idem |
| SLH-DSA-SHA2-192s / 256s | 3 / 5 | 16.224 / 29.792 | 48 / 64 | solo hash, sin estado | FIPS 205 | idem |
| Falcon-512 (FN-DSA-512) | 1 (según implementadores) | 666 (padded; 752 máx. sin padding) | 897 | retículos NTRU | FIPS 206 **sin publicar, ni borrador**, a 30-09-2026 (acta del TSC de Open Quantum Safe del 01-09-2026, con Pornin) | ⚠️ **sin fuente primaria**: `api.h` de PQClean y README de `rust-fn-dsa`; la especificación de Falcon no era alcanzable |
| XMSS-SHA2_10_256 | n = 32 | 2.500 | 68 📐 | solo hash, **con estado** | RFC 8391, SP 800-208 | RFC 8391, Tabla 3 (espejo); la clave, `OID ‖ root ‖ SEED`, derivada |
| XMSSMT-SHA2_40/8_256 (la casa) | n = 32 | **18.469** | 68 📐 | solo hash, **con estado** | RFC 8391, SP 800-208 | RFC 8391, Tabla 5 (espejo); ✅ medido en §1 |
| LMS_SHA256_M32_H10 / W8 | n = 32 | 1.452 📐 | 56 📐 | solo hash, **con estado** | RFC 8554, SP 800-208 | derivado de RFC 8554 (§4.1, `4 + n·(p+1)` con p = 34, más `4 + n·h`); la única cifra literal del RFC es la Tabla 3 de HSS: 1.616 B con h = 15 |
| **uov-Is** (n=160, m=64, GF(16)) | 1 | **96** | 66.576 (412.160 clásica) | multivariante | ronda 3, parámetros sin cambio | README de `pqov`, rama de ronda 3 |
| uov-Ip (ronda 3: n=119, m=45) | 1 | 135 | 46.591 (321.300 clásica) | multivariante | ronda 3; en ronda 2 era 128 B y los ataques wedge/intersección lo bajaron a ~128 bits | idem |
| MAYO_2 (ronda 3) | 1 | 239 | 2.928 | multivariante | ronda 3; en ronda 2 era 186 B | README de `MAYO-C` |
| MAYO_1 (ronda 3) | 1 | 464 | 1.456 | multivariante | ronda 3 | idem |
| SNOVA_I_S (ronda 3) | 1 | 272 | 1.016 | multivariante | ronda 3; los conjuntos de ronda 2 fueron sustituidos | paquete `SNOVA_Round3` |
| QR-UOV I, L=3 (ronda 3) | 1 | 200 | 24.256 | multivariante | ronda 3 | macros de `qruov/round3` |
| SQIsign_p324_3 (ronda 3, nivel I) | 1 | **200** | 83 | isogenias | ronda 3; etiqueta `nist-v3` del 01-09-2026, primos nuevos tras el ataque en p^(1/3) de Wesolowski (ePrint 2026/1486) | `api.h` y KAT del repositorio oficial `SQIsign/the-sqisign` |
| SQIsign_lvl1 (ronda 2) | 1 | 148 | 65 | isogenias | sustituido por la ronda 3 | idem, etiqueta `nist-v2` |
| SQIsign, formato compacto de dimensión 4 | 1 | 142 | 84 | isogenias | propuesta de terceros (ePrint 2026/2221), no es el formato de la especificación; verificar cuesta unas diez veces más | README de `sqisign-rs` |
| HAWK-512 | 1 | 555 | 1.024 | retículos (isomorfismo de retículos) | ronda 3; **retirado por su equipo el 29-07-2026** tras una recuperación de clave (ePrint 2026/1593, Straznickas y Weis); la ronda 3 queda sin retículos | `hawk.h` del repositorio `hawk-sign/dev`; el retiro, por el pqc-forum (no leído en origen) |
| Rainbow-I (ronda 3, 2020) | 1 | **66** | 161.600 (60.192 comprimida) | multivariante | **roto**: Beullens, ePrint 2022/214, «53 hours (one weekend) of computation time on a standard laptop» para los parámetros SL 1 de la ronda 2; NIST no lo seleccionó (IR 8413, no leído) | tamaños compilando el código de referencia de la ronda 3, reproducidos dos veces; el resumen del ataque, en copia de terceros |
| GeMSS128 (ronda 3, 2020) | 1 | **33** | 352.188 | multivariante (HFEv-) | **roto**: Tao, Petzoldt y Ding, ePrint 2020/1424 (Crypto 2021); no continuado por NIST | tamaños de las cabeceras de SUPERCOP (no del paquete oficial); el artículo, en conversión de terceros |
| VDOO (2023; NGCC v1.1, 2026) | 1 | 96 / **85** | sin dato / 330.855 | multivariante | propuesta académica, hoy candidata del concurso chino NGCC; **criptoanalizada en septiembre de 2026** (ePrint 2026/2178, «all proposed parameters fail to meet their claimed NIST security levels», por resumen; y dos hallazgos «Confirmed» en ngcc.dev) | resumen de arXiv 2312.09535 (espejo) y especificación v1.1 en el arnés del NGCC |

Tres lecturas, en orden de importancia:

**Censo de lo que baja de 100 bytes**, con todo lo que se encontró, para que no haya que
rehacerlo: `uov-Is` 96 B (vigente, ronda 3); VDOO 96 y 85 B (criptoanalizada este mes); TUOV-Is
80 B (ronda 1 de NIST, no avanzó, clave de 65.552 B); KAZ-SIGN 74 B (solo en `nist-sigs-zoo`, sin
fuente primaria), Facto-DSA 40 B y DME-Sign 32 B (rotos por forja); Rainbow-I 66 B y GeMSS128 33 B (rotos por recuperación de clave). **Uno
vivo de nueve**, y con 66 KB de clave pública. No se revisaron a fondo las propuestas KpqC ni
HPPC/3WISE.

1. **El listón real no son los 2.420 bytes de ML-DSA-44.** Falcon-512 lleva años en 666 B y
   ML-DSA-44 ni siquiera es categoría 1, es 2. Comparar contra Dilithium infla el mérito de
   cualquier cosa que se proponga.
2. **Lo único vivo por debajo de 100 bytes es `uov-Is`, a 96 B, con una clave pública de 66 KB.**
   Es multivariante: la familia de Rainbow y GeMSS, los dos esquemas que bajaron de 100 B dentro
   del concurso de NIST y se rompieron. UOV lleva en pie desde 1999, pero en septiembre de 2026
   tres de las cuatro multivariantes de la ronda 3 cambiaron de parámetros por ataques nuevos
   (`uov-Ip`, MAYO, SNOVA), y sigue saliendo criptoanálisis con fecha de este mismo mes. Adoptarla
   sería revocar la decisión de una sola familia de supuestos (`AUDITORIA.md` §106), y no
   inventaríamos nada: usaríamos UOV.
3. ⚠️ **Una premisa de la entrada 87 del `BACKLOG.md` estaba mal.** Su tabla decía que SLH-DSA
   firma «decenas de KB». FIPS 205 dice 7.856 B para SHA2-128s y 16.224 B para 192s: **las dos por
   debajo de los 18.469 B de la XMSS^MT que la casa firma hoy**, sin estado y con el mismo supuesto
   (solo hash). Solo el conjunto de categoría 5, 29.792 B, es mayor. No decide nada por sí sola
   —firmar con SLH-DSA-s es lento, y **no se ha medido aquí**—, pero cambia el precio que la 87 daba
   por sabido: sin estado ya no cuesta «decenas de KB», cuesta menos que hoy.

⚠️ **Criptoanálisis de 2025-2026 citado por resumen, no leído en fuente** (eprint bloqueado desde
esta sesión): ePrint 2025/1143 (Ran, wedge sobre UOV), 2026/298 (Furue-Ikematsu, intersección),
2026/237 (Bros et al., SNOVA), 2026/2154 (Beullens-Hess, SNOVA ronda 3), 2026/2181 (Merz-Ran,
Schur-Macaulay sobre toda la ronda 2 de UOV, MAYO, SNOVA y QR-UOV), 2026/2247 (Ostuzzi, MAYO ronda
3, publicado el 30-09-2026, que según su resumen deja MAYO_2 tres bits por debajo de la categoría
1). De Wesolowski, ePrint 2026/1486 (el problema de la isogenia supersingular en tiempo y memoria
p^(1/3+o(1))), se leyó una copia literal de terceros: dice que **no rompe** SQIsign, y el equipo
cambió los primos el 01-09-2026. Se listan para que quien coteje sepa dónde mirar, no como hechos verificados aquí.

---

## 4. Lo que compran los árboles de Merkle, y por qué no es una firma 📖 ✅

Los Merkle Tree Certificates del IETF (`draft-ietf-plants-merkle-tree-certs`, versión 06 del
21-09-2026, grupo PLANTS, Standards Track; leído en la copia publicada por el propio grupo en
GitHub) sustituyen la firma del certificado por una **prueba de inclusión** en un registro de
emisión, y mueven la firma a otro sitio: la CA firma *checkpoints* y subárboles, y unos
*cosigners* independientes los cofirman. Sus cifras (§6.5 del borrador, SHA-256):

| certificado | lote | prueba | y además |
|---|---|---|---|
| standalone, checkpoint cada 2 s | ≈ 3.000 | 12 hashes, **384 B** | las cofirmas de la CA y del quórum, «a sufficient set of signatures», **no contadas** |
| landmark-relative, landmark cada hora | ≈ 5.400.000 | 23 hashes, **736 B** | ninguna firma; exige que el verificador **ya tenga el subárbol** |
| cota del borrador | hasta 2³² | 32 hashes, 1.024 B | |

Y la cabeza no viaja con el certificado: «landmark subtrees are predistributed to relying parties
… separate from the application protocol» (§3); cómo, «This document does not prescribe» (§7.4:
un servicio de actualización del navegador). El verificador guarda del orden de 338 subárboles,
unos 10.816 B. El certificado sin firmas «only work[s] in a sufficiently up-to-date relying
party», y el borrador manda desplegar el otro al lado (§6.4).

Es exactamente la construcción de la casa: **cabeza firmada + camino + cofirmas**. Medido en §1, el
coste marginal por elemento es `32 · h` bytes:

| altura `h` | elementos por cabeza | camino | quién lo paga hoy en la casa |
|---|---|---|---|
| 32 | 2³² | 1.024 B | el acuse y el recibo (`RECEP_DEPTH`) |
| 63 | 2⁶³ | 2.016 B | el árbol de consumos |
| 3 | 8 | 96 B | nadie: sería un lote de ocho |

Para bajar de 100 bytes hacen falta **h ≤ 3** y que el verificador **ya tenga la cabeza** (el
testigo la tiene: la fija la primera vez que la ve, `AUDITORIA.md` §245). Y entonces lo que tiene
en la mano no es una firma: es **un recibo de inclusión bajo una cabeza que otro firmó con 18.469
bytes**, amortizados entre los ocho. Los MTC no acortan una firma: la **amortizan**, y reintroducen
el árbol que la pregunta quería evitar.

---

## 5. Veredicto

| condición | con la familia del hash | con otra familia |
|---|---|---|
| < 100 bytes | ✘ aritmética de §2: ≥ 2⁸⁵ hashes por cadena | ✔ solo `uov-Is`, 96 B, clave de 66 KB; SQIsign se queda en 200 |
| sin perder seguridad | ✘ solo bajando el resumen a 64 bits poscuánticos | ⚠️ familia con dos rupturas y parámetros cambiados este mes |
| sin árbol de estado | ✔ SLH-DSA, a 7.856 B; STARK, a 43 KB | ✔ |
| las tres a la vez | **✘** | **✘ sin revocar §106** |

**No es alcanzable con las herramientas ni con los supuestos de la casa**, y este documento existe
para que la pregunta no vuelva a hacerse sin este número delante.

---

## 6. Lo que sí es publicable, por su nombre

1. **Este documento.** Un resultado negativo con su aritmética reproducible y sus cifras
   contrastadas cumple «medir antes que afirmar» (`CONTRIBUTING.md` §1) igual que uno positivo.
2. **Recibos cortos bajo cabeza firmada, como medida.** La tabla de §4 con los `32 · h` de la casa
   y la firma amortizada. Se publica como lo que es: coste amortizado de una XMSS con la cabeza como
   ancla y el testigo como verificador que ya la tiene. Nunca como «firma nueva».
3. **La corrección a la entrada 87** (§3, lectura 3): SLH-DSA-SHA2-128s pesa 7.856 B, no «decenas
   de KB». Es una premisa de la decisión pendiente sobre agilidad criptográfica, y hay que
   re-medirla antes de decidir: el tiempo de firma de SLH-DSA-s sobre el hardware real es la
   cifra que falta, con el método de §263.

Y dos vías que **no** son de este proyecto, declaradas para que no vuelvan disfrazadas:

- **Revelación diferida al ritmo del latido** (un MAC de 32 B cuya clave se publica en la cabeza
  siguiente). Solo hash, sin árbol, bajo 100 bytes; pero es otro modelo de seguridad: exige
  sincronía y no da no repudio por sí sola. Publicarla como firma sería la afirmación categórica
  que `SECURITY.md` §5 prohíbe.
- **Cambiar de familia** (UOV, SQIsign). Requiere revocar §106 y asumir claves públicas de decenas
  de KB o supuestos con criptoanálisis abierto.

---

## 7. Lo que este documento NO afirma

- No afirma que una firma poscuántica de menos de 100 bytes sea imposible en general. Afirma que
  no lo es con la familia de supuestos de la casa y con las construcciones conocidas, con las
  cifras leídas a 30-09-2026.
- No mide el tiempo de ningún esquema ajeno. Las cifras de §3 son tamaños leídos, no medidos.
- No propone cambiar §106 ni el conjunto XMSS elegido en `doc/xmss-evaluacion.md`.
- No ha cotejado las copias «espejo» con el original de NIST o del IETF: dos espejos idénticos
  reducen el riesgo, no lo eliminan.

---

## 8. Pendientes que deja

1. Cotejar cada «espejo» con el original cuando la red lo permita (SHA-256 de los PDF de FIPS 204
   y 205; texto de RFC 8391 y 8554 y sus erratas; NIST IR 8610 para la lista de la ronda 3; la
   especificación de Falcon, que hoy no tiene fuente primaria en esta tabla).
2. Entrada 87: re-medir el precio de «sin estado» con SLH-DSA-SHA2-128s y 192s sobre el hardware
   real, tiempo de firma incluido, antes de la decisión.
3. Si alguien quiere el recibo de lote (§6, 2): es un banco, no un diseño; empieza por medir cuánto
   pesa el sobre entero con `h = 3` frente al de `h = 32`.

## Fuentes

- FIPS 204 (ML-DSA) y FIPS 205 (SLH-DSA), 13-08-2024: `https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.204.pdf`,
  `https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.205.pdf` (espejo: copias idénticas en
  `github.com/Cosmian/standards` y `github.com/hzhang092/nist-pqc-rag-agent`).
- RFC 8391 (XMSS) y RFC 8554 (LMS): `https://www.rfc-editor.org/rfc/rfc8391.txt`,
  `https://www.rfc-editor.org/rfc/rfc8554.txt` (espejo: `mnot/rfc-refs` y `mcpherrinm/rfc-mirror`,
  md5 idénticos). NIST SP 800-208 (octubre de 2020), leído en una conversión a texto de terceros.
- Falcon / FN-DSA: `https://github.com/PQClean/PQClean` (`crypto_sign/falcon-padded-512/clean/api.h`)
  y `https://github.com/pornin/rust-fn-dsa` (nota del 22-07-2026 sobre FIPS 206).
- UOV: `https://github.com/pqov/pqov` (README, ronda 3). MAYO: `https://github.com/PQCMayo/MAYO-C`
  (README, ronda 3 y etiqueta de ronda 2). SNOVA: `https://github.com/PQCLAB-SNOVA/NIST_Submissions`
  (`SNOVA_Round3.zip`). QR-UOV: `https://github.com/qruov/round3`.
- SQIsign: `https://github.com/SQIsign/the-sqisign` (etiquetas `nist-v2` y `nist-v3`, `src/nistapi/*/api.h` y `KAT/`);
  `https://github.com/anchorageoss/sqisign-rs` (formato compacto, tercero).
- Merkle Tree Certificates: `https://datatracker.ietf.org/doc/draft-ietf-plants-merkle-tree-certs/` (espejo:
  `https://ietf-plants-wg.github.io/merkle-tree-certs/`, etiqueta 06 y copia del editor del 30-09-2026).
- Rainbow y GeMSS: código de referencia de la ronda 3 y cabeceras de SUPERCOP (tamaños compilados);
  Beullens, ePrint 2022/214, y su repositorio `https://github.com/WardBeullens/BreakingRainbow`; Tao,
  Petzoldt y Ding, ePrint 2020/1424 (conversión de terceros).
- HAWK: `https://github.com/hawk-sign/dev` (`src/hawk.h`); retiro anunciado en el pqc-forum de NIST el
  29-07-2026, citado por `https://github.com/PQShield/nist-sigs-zoo` (`history.yaml`).
- VDOO: arXiv 2312.09535 (resumen, espejo) y `sign-33-spec.pdf` en `https://github.com/ngcc-dev/ngcc-harness`;
  ePrint 2026/2178 por resumen. TUOV, KAZ-SIGN, DME-Sign y Facto-DSA: `nist-sigs-zoo` y los informes de
  `ngcc.dev`, no sus especificaciones.
- Pruebas sucintas: Groth16 en `https://github.com/zcash/zips` (`protocol/protocol.tex`); LaBRADOR ePrint 2022/1341,
  Greyhound 2024/1293, WHIR 2024/1586, Aurora 2018/828, LUNA+ 2026/1639, ISW21 2021/977 y Mathialagan, Peters y
  Vaikuntanathan 2024/227, leídos en las conversiones de `https://github.com/BaDaaS/cryptography.academy` (espejo) o,
  donde se dice «resumen», solo en su resumen.
