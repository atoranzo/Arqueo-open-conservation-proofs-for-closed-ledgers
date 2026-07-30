#!/usr/bin/env python3
"""Entrada 34: redacta el limite de grados en depuracion (decidido en §46)
como seccion propia §20-bis y en el README. Corrige de paso la cifra
172->174 y el comentario contradictorio de allocate_pending."""
import hashlib
def md5(p): return hashlib.md5(open(p,'rb').read()).hexdigest()
ESPERADOS = {
 'AUDITORIA.md':'41f9f797e0414eda9e5f02c0377c90d6',
 'BACKLOG.md':'d3fb4ffa2aa52b8b38ddd53aeae9af78',
 'README.md':'2f09cc82194fab012ddb72a11c55131d',
 'crates/zk-ssl/src/two_phase.rs':'39c6fd652e936ba711632e637215b0a7',
}
for f,h in ESPERADOS.items():
    real = md5(f)
    assert real==h, f"{f}: md5 {real}, esperado {h} - NO se toca nada"
def sub(path, old, new, n=1):
    c = open(path).read()
    assert c.count(old)==n, f"{path}: ancla hallada {c.count(old)} veces:\n{old[:70]}"
    open(path,'w').write(c.replace(old,new,n))

# 1. cifra rancia 172 -> 174 en la tabla de §20
sub('AUDITORIA.md',
"| `zk-ssl --release` | \u2705 **172 pasan** |",
"| `zk-ssl --release` | \u2705 **174 pasan** |")
# y en el bloque de ejemplo de §20 (el que ilustra el fallo de check_figures)
sub('AUDITORIA.md',
"cargo test -p zk-ssl --release        # la capa: 172 tests",
"cargo test -p zk-ssl --release        # la capa: 174 tests")

# 2. la declaracion del limite, como cierre de §20
ANCLA_20 = """**Reparto de los 65**: 48 en `tests`, 9 en `iso`, 4 en `snapshot`, 3 en
`metrics`, 1 en `client`."""
DECL = ANCLA_20 + """

### El limite, declarado (entrada 6/24/25, decidido en §46)

Estos 65 fallos **no son un defecto de solidez, y no se van a arreglar**.
La razon, y por que esa decision es la correcta:

Winterfell comprueba en depuracion que el grado declarado de cada
restriccion se **realice** en la traza concreta que se prueba. Una
restriccion cuyo grado **depende del valor del testigo** viola esa
comprobacion en los testigos donde el grado colapsa, aunque sea
perfectamente valida. En esta capa eso ocurre en dos familias:

- **Bits de camino de Merkle** (arboles de cuentas, pendientes y
  congelados). Una posicion baja tiene los bits altos a cero, y una
  restriccion booleana `bit \u00d7 (bit \u2212 1)` sobre una columna constante-cero
  tiene grado cero en vez del declarado (§35, §37.7).
- **Margenes que pueden ser cero** por diseno: el margen del tope de
  emision cuando se emite exactamente hasta el tope, y la diferencia de la
  comprobacion de rango cuando `amount == balance`, que el circuito de
  cumplimiento **necesita** (§37.2, caso B).

**En release \u2014el modo de produccion\u2014 winterfell no comprueba grados**, y las
pruebas se generan y verifican correctamente en ambos casos. La comprobacion
de grados en depuracion es una red *adicional*, util donde aplica, y esta
clase de restriccion queda fuera de su alcance por naturaleza, no por un
fallo del circuito.

**Por que se declara y no se corrige** (§46): la unica forma de que los bits
de camino no colapsen es no asignar las posiciones bajas, y como
`allocate_pending` reutiliza huecos (§46.1), eso obliga a **migrar los
pendientes vivos** de los ledgers existentes \u2014mover valor en transito, del
peso de §36\u2014 para arreglar una comprobacion que el modo de produccion no
necesita. Es desproporcionado. Y para los margenes de dominio (§37.2 caso B)
no hay arreglo posible: el valor cero es legitimo.

> **Limite conocido, no fallo.** El proyecto usa release como modo de
> produccion y lo documenta. Perseguir el 100 % de tests en depuracion
> costaria una migracion de fondos o seria imposible, y compraria una
> comprobacion redundante con release. Se elige coherencia sobre
> completitud."""
sub('AUDITORIA.md', ANCLA_20, DECL)

# 3. README: nota junto al bloque que menciona los 65
sub('README.md',
"""> `cargo test` sin \u00e9l **falla en 65 tests de `zk-ssl`**, y no porque el""",
"""> \u26a0\ufe0f **Estos 65 fallos son un limite conocido, no un defecto**, y estan
> declarados en `AUDITORIA.md` \u00a720: winterfell comprueba grados solo en
> depuracion, y ciertas restricciones tienen grado que depende del testigo
> (bits de camino de Merkle, margenes que pueden ser cero). En release \u2014el
> modo de produccion\u2014 las pruebas se generan y verifican bien. No se
> corrige porque hacerlo exigiria migrar fondos en transito, y release no
> lo necesita.
>
> `cargo test` sin \u00e9l **falla en 65 tests de `zk-ssl`**, y no porque el""")

# 4. el comentario contradictorio de allocate_pending
sub('crates/zk-ssl/src/two_phase.rs',
"""    /// Y las posiciones **ya se liberaban**: `apply_claim` pone la hoja a
    /// cero al cobrarse el pendiente. Nadie las reutilizaba.""",
"""    /// Y las posiciones **ya se liberaban**: `apply_claim` pone la hoja a
    /// cero al cobrarse el pendiente, y el bucle de abajo **las reutiliza**:
    /// devuelve el primer hueco libre desde cero antes de `next_pending`.
    /// (Una version anterior de este comentario decia «Nadie las
    /// reutilizaba», contradiciendo al parrafo siguiente y al codigo.)""")

# 5. cerrar la 34 en el backlog
sub('BACKLOG.md',
"""- [ ] **34. Redactar el limite de grados en depuracion.** La decision de
  la 6/24/25 esta tomada (§46): un parrafo en README y AUDITORIA §20 que
  declare que winterfell-depuracion es incompatible con grados dependientes
  del testigo, que no es fallo de solidez, y que release es produccion.
  Cierra el frente de grados sin migrar nada.""",
"""- [x] **34. Redactar el limite de grados en depuracion.** ~~La decision de
  la 6/24/25 esta tomada (§46).~~ **Hecho** el 30-07-2026: declarado en
  AUDITORIA §20 y en el README como limite conocido, con corregida de paso
  la cifra 172->174 y el comentario contradictorio de `allocate_pending`.
  Con esto el **frente de grados (6, 24, 25, 34) queda cerrado**: decidido y
  documentado, sin migrar nada.""")

sub('BACKLOG.md',
"""**Estado**: 25 abiertas, 9 resueltas. Ultima revision: 30 de julio de 2026.
El frente de grados (6, 24, 25) tiene **decision tomada** (§46): declarar,
no migrar. Queda redactarla (34).""",
"""**Estado**: 24 abiertas, 10 resueltas. Ultima revision: 30 de julio de 2026.
El frente de grados (6, 24, 25, 34) queda **cerrado** (§46, §20): declarado
como limite conocido de winterfell, sin migrar. No es fallo de solidez.""")

print("OK - entrada 34 hecha, limite declarado, 172->174, comentario corregido")
for f in sorted(ESPERADOS):
    print(f"  {md5(f)}  {f}")
