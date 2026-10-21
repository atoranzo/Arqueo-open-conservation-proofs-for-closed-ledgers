#!/usr/bin/env python3
# ── LAS RAICES DE PRODUCCION, SIN CURVAS (§788, ARQ-08 P1) ──────────
# La familia de supuestos de la casa es la resistencia del hash (§106): STARK/FRI, Rescue, Blake3,
# Merkle y XMSS. Dos excepciones declaradas: ML-DSA-44 en el medio del ancla (§705, RFC-0013 D-C),
# que no es una curva; y el transporte TLS del cli y del SDK (§838): `rustls` sobre `ring`, por
# `ureq`. ⚠️ §835: el VERDE decia «ninguna curva en la clausura de produccion»; el TLS de `ureq`
# enlaza `ring` en el cli y en el SDK, y `ring` no esta en el patron (la 122 de BACKLOG.md).
# ⚠️ §838: decia que la unica excepcion era ML-DSA-44; `ring` ya esta en el patron, con R4 y R5.
# Los comparativos del workspace -Groth16, Nova, Plonk, halo2- traen
# curvas al Cargo.lock, y hasta el §788 que no llegaran a los binarios de produccion lo sostenia la
# disciplina: un `use ark_ff` en el nodo compilaba (doc/blueprint-v2.md, ARQ-08). Reglas, declaradas
# ANTES de mirar:
#
#  R1  RAICES: los cuatro binarios y bibliotecas que un operador o un tercero ejecutan -el nodo,
#      el testigo (zk-ssl-cli), el SDK y el kit-. Su clausura es la de `cargo tree --locked
#      --offline -e normal`: lo que se enlaza, sin dependencias de desarrollo ni de compilacion.
#      ⚠️ §838: decia eso sin `--all-features` ni `--target all`: R1 y R2 no veian una dependencia
#      opcional tras una feature de la raiz que no va por defecto, ni una de otro target; ahora si.
#  R2  NINGUN crate de esa clausura casa con CURVAS: las bibliotecas de curvas elipticas y de
#      emparejamientos conocidas (arkworks, BLS12-381, BN254, Pasta, curve25519, secp256k1, P-256...),
#      y las del TLS (`ring`, `aws-lc-*`, `rustls`, `rustls-webpki`, `webpki`), salvo R4.
#  R3  PRUEBA DE VIDA: cada raiz tiene al menos MINIMO crates en su clausura (una clausura vacia es un
#      instrumento ciego), el patron reconoce los nombres de su autotest, la regla de R4 y la busqueda
#      de R5 dan ROJO en sus autotests, y R5 lee al menos MINIMO_FUENTES ficheros: si algo se rompe, la
#      puerta no puede salir verde por no reconocer nada. Y FALTA (R1) casa solo con FALTA_SI (§843).
#  R4  EL TRANSPORTE, POR ARISTAS (§838): solo en las raices de TRANSPORTE_RAICES, y cada crate de
#      TRANSPORTE solo con los padres que dice: `ureq` bajo el cli o el SDK; `rustls` bajo `ureq`;
#      `rustls-webpki` bajo `rustls`; `ring` bajo `rustls` o `rustls-webpki`. `aws-lc-*`, nunca.
#  R5  NADIE MAS LO USA: en los `.rs` de `src/` de cada crate de la casa de la clausura (los que
#      `cargo tree` da con ruta dentro del repositorio), sin comentarios, ningun `ring::` ni ningun
#      `rustls` o `webpki` como palabra entera: cubre `ureq::rustls`, `rustls::` y `webpki::`.
#
# Ceguera declarada: mira NOMBRES de crate y, en R5, texto; una curva escrita a mano dentro de un
# crate de la casa, o en un crate con otro nombre, no se ve. R5 no lee fuera de `src/` (`examples/`,
# `tests/`, `build.rs`, ni un `#[path]` o un `include!` que salga de `src/`), ni ve un nombre que
# fabrique una macro. `--all-features` activa solo las features de la raiz: una de una dependencia
# que la raiz no pide no sale, aunque un build de la raiz la pueda activar (`--features
# zk-ssl-medio/firmar`, o por unificacion con otro paquete del build). Y no mira los comparativos:
# que sigan en el lock es la parte P2/P3 de ARQ-08, que no esta adoptada.
import os, re, subprocess, sys

RAIZ = os.environ.get('CANON_RAIZ') or os.path.normpath(
    os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
RAICES = ('zk-ssl-node', 'zk-ssl-cli', 'zk-ssl-sdk', 'zk-ssl-verify')
MINIMO = 50
MINIMO_FUENTES = 150
CURVAS = re.compile(r'^(ark-.*|bls12[-_]381.*|blst|bn254.*|ark_bn254|pairing|halo2curves|pasta_curves|'
                    r'jubjub|curve25519-dalek.*|ed25519-dalek|x25519-dalek|k256|p256|p384|p521|'
                    r'secp256k1.*|libsecp256k1.*|elliptic-curve|primeorder|group|ff|ec-gpu.*|'
                    r'ring|aws-lc-.*|rustls|rustls-webpki|webpki)$')
AUTOTEST_SI = ('ark-ff', 'ark-bls12-381', 'bls12_381', 'curve25519-dalek', 'k256', 'secp256k1', 'pairing', 'group',
               'ring', 'aws-lc-rs', 'aws-lc-sys', 'rustls', 'rustls-webpki', 'webpki')
AUTOTEST_NO = ('winterfell', 'blake3', 'sha2', 'ml-dsa', 'xmss', 'sled', 'zk-ssl-hash', 'groupy-no',
               'rustls-pki-types', 'webpki-roots', 'untrusted', 'ureq', 'ringbuf')
# R4: el crate, y los padres con los que se admite. Nada mas del patron se admite en ninguna parte.
TRANSPORTE_RAICES = ('zk-ssl-cli', 'zk-ssl-sdk')
TRANSPORTE = {'ureq': TRANSPORTE_RAICES, 'rustls': ('ureq',), 'rustls-webpki': ('rustls',),
              'ring': ('rustls', 'rustls-webpki')}
# R5: `ring::` (con o sin espacios antes de `::`), o `rustls` o `webpki` como palabra entera.
USO_TLS = re.compile(r'(?<![A-Za-z0-9_])(?:ring\s*::|rustls(?![A-Za-z0-9_])|webpki(?![A-Za-z0-9_]))')
CRUDA, IDENT = re.compile(r'r(#*)"'), re.compile(r'[A-Za-z0-9_]')
ARBOL_SI = '0zk-ssl-cli v0 (/r/crates/zk-ssl-cli)\n1ureq v2\n2rustls v0\n3ring v0\n3rustls-webpki v0\n4ring v0 (*)\n'
ARBOL_NO = (  # cada uno, con al menos un fallo de R2 o R4
    ('zk-ssl-cli', ARBOL_SI + '1ring v0 (*)\n'),
    ('zk-ssl-cli', ARBOL_SI + '1hyper-rustls v0\n2rustls v0 (*)\n'),
    ('zk-ssl-cli', ARBOL_SI.replace('3ring v0\n', '3aws-lc-rs v1\n')),
    ('zk-ssl-cli', ARBOL_SI + '1zk-ssl-medio v0 (/r/crates/zk-ssl-medio)\n2ureq v2 (*)\n'),
    ('zk-ssl-node', '0zk-ssl-node v0 (/r/crates/zk-ssl-node)\n1zk-ssl-sdk v0 (/r/crates/zk-ssl-sdk)\n2ureq v2\n'
                    '3rustls v0\n4ring v0\n'),
)
FUENTE_SI = ('use ureq::rustls::crypto::ring;', 'let k = webpki::ring::ED25519;', 'use ureq::{json, rustls};',
             'use ureq::rustls as r;', 'let x = ring :: agreement::X25519;', 'let s = "//"; use ring::rand;',
             "let c = '\"'; use ring::rand;", 'let s = r#"/*"#; use ring::rand;',
             'let s = cr"\\"; let t = "//"; use ring::rand;')
FUENTE_NO = ('let s = String::from("x");', 'o == Ordering::Less', '// ureq::rustls::crypto::ring',
             '/* ring:: /* anidado */ webpki:: */', '//! `ring` por `rustls`', 'use ringbuf::X;',
             'let q = spring::a(); let w = rustls_x;', 'let s = r"\\"; // ring:: "', "let c = '\"'; // ring:: \"",
             "fn f<'a>(x: &'a str) -> &'a str { x } // ring::")
# R1, cuando `cargo tree` no puede leer el arbol porque a la cache de cargo le falta un fuente (§843):
# no es una curva, y el remedio es otro. Cargo nombra solo el primero que falta. FALTA casa solo si
# la causa que sigue a «failed to download» es la de `--offline`: un `.crate` roto en la cache da el
# mismo encabezado con otra causa, y `cargo fetch --locked` no lo arregla. El autotest de R3: FALTA
# saca de FALTA_SI exactamente su crate, y no casa con ninguno de FALTA_NO: un `Cargo.lock` que pide
# cambios con `--locked`, y un `.crate` truncado en la cache.
FALTA = re.compile(r'failed to download `([^`]+)`\s+Caused by:\s+attempting to make an HTTP request, '
                   r'but --offline was specified')
FALTA_SI = ('error: failed to download `anstyle-wincon v3.0.11`\n\nCaused by:\n  attempting to make an HTTP '
            'request, but --offline was specified\n', 'anstyle-wincon v3.0.11')
FALTA_NO = ('error: the lock file /r/Cargo.lock needs to be updated but --locked was passed to prevent this\n',
            'error: failed to download `ureq v2.12.1`\n\nCaused by:\n  unable to get packages from source\n\n'
            'Caused by:\n  failed to unpack package `ureq v2.12.1`\n')


def sin_comentarios(texto):
    """El fuente con sus comentarios -de linea, de bloque y anidados- en blanco, sin mover lineas.
    Las cadenas, con o sin `r#` (tambien `br` y `cr`), y los caracteres se saltan enteros, y no abren
    comentario."""
    out, i, n = [], 0, len(texto)
    while i < n:
        c = texto[i]
        if texto.startswith('//', i):
            j = texto.find('\n', i)
            j = n if j < 0 else j
            out.append(' ' * (j - i)); i = j
        elif texto.startswith('/*', i):
            j, nivel = i + 2, 1
            while j < n and nivel:
                if texto.startswith('/*', j): nivel += 1; j += 2
                elif texto.startswith('*/', j): nivel -= 1; j += 2
                else: j += 1
            out.append(re.sub(r'[^\n]', ' ', texto[i:j])); i = j
        elif c == 'r' and CRUDA.match(texto, i) and not IDENT.match(
                texto[max(0, i - 2):i - 1] if texto[i - 1:i] in ('b', 'c') else texto[i - 1:i]):
            m = CRUDA.match(texto, i)
            fin = texto.find('"' + m.group(1), i + len(m.group(0)))
            j = n if fin < 0 else fin + 1 + len(m.group(1))
            out.append(texto[i:j]); i = j
        elif c == '"':
            j = i + 1
            while j < n and texto[j] != '"':
                j += 2 if texto[j] == '\\' else 1
            out.append(texto[i:j + 1]); i = j + 1
        elif c == "'" and (texto.startswith("\\", i + 1) or texto.startswith("'", i + 2)):
            j = texto.find("'", i + 3 if texto.startswith("\\", i + 1) else i + 2)
            j = n if j < 0 else j + 1
            out.append(texto[i:j]); i = j
        else:
            out.append(c); i += 1
    return ''.join(out)


def arbol(raiz):
    # `--all-features` y `--target all` (§838): sin ellas, `cargo tree` da solo las features por
    # defecto y el target de la maquina, y una dependencia opcional tras otra feature, o una de
    # `[target.'cfg(windows)'.dependencies]`, no salia, y R2 y R4 no la veian. Con ellas, el arbol
    # es la union de lo que enlaza un build de la raiz con sus propias features, en cualquier
    # target. No trae una feature de una dependencia que la raiz no pide (`--features
    # zk-ssl-medio/firmar`, o la unificacion con otro paquete del build): la ceguera de la cabecera.
    # Las dos opciones valen con `--locked --offline`. ⚠️ §843: decia solo eso; valen sin red si la
    # cache de cargo tiene el fuente de cada crate de la clausura en todos los targets. Un `cargo
    # build` baja solo los de la maquina, y en un clon recien hecho R1 salia ROJO en las cuatro raices
    # con el ultimo renglon de cargo, «attempting to make an HTTP request, but --offline was
    # specified», como si la puerta hubiera visto algo. Ahora main() reconoce ese fallo (FALTA), lo
    # dice, y da el remedio: `cargo fetch --locked`, una vez y con red, que baja todos los targets.
    r = subprocess.run(['cargo', 'tree', '--locked', '--offline', '-q', '-p', raiz, '-e', 'normal',
                        '--all-features', '--target', 'all', '--prefix', 'depth'],
                       cwd=RAIZ, capture_output=True, text=True)
    if r.returncode != 0:
        return None, r.stderr
    return r.stdout, None


def leer_arbol(texto):
    """Del `cargo tree --prefix depth`: los crates, las aristas (padre, hijo) y la ruta de los locales."""
    crates, aristas, rutas, pila = set(), set(), {}, []
    for l in texto.splitlines():
        m = re.match(r'^(\d+)(\S+) v\S+(?: \((/[^)]*)\))?', l)
        if not m:
            continue
        nivel, nombre = int(m.group(1)), m.group(2)
        del pila[nivel:]
        if pila:
            aristas.add((pila[-1], nombre))
        pila.append(nombre)
        crates.add(nombre)
        if m.group(3):
            rutas[nombre] = m.group(3)
    return crates, aristas, rutas


def reglas(raiz, crates, aristas):
    """R2 y R4 sobre la clausura y las aristas de una raiz."""
    fallos = ['R2 %s enlaza %s, de las bibliotecas de curvas: la familia de supuestos de produccion es el hash (§106)'
              % (raiz, c) for c in sorted(crates) if CURVAS.match(c) and c not in TRANSPORTE]
    for padre, hijo in sorted(aristas):
        if hijo not in TRANSPORTE:
            continue
        if raiz not in TRANSPORTE_RAICES:
            fallos.append('R4 %s enlaza %s (bajo %s): el transporte TLS solo se admite en %s (§838)'
                          % (raiz, hijo, padre, ' y '.join(TRANSPORTE_RAICES)))
        elif padre not in TRANSPORTE[hijo]:
            fallos.append('R4 %s: %s cuelga de %s, y solo se admite bajo %s (§838)'
                          % (raiz, hijo, padre, ' o '.join(TRANSPORTE[hijo])))
    return fallos


def usos_tls(texto):
    limpio = sin_comentarios(texto)
    return [(limpio.count('\n', 0, m.start()) + 1, m.group(0)) for m in USO_TLS.finditer(limpio)]


def main():
    rojo = []
    for n in AUTOTEST_SI:
        if not CURVAS.match(n):
            rojo.append('R3 el patron no reconoce %s, que es una curva: la puerta esta ciega' % n)
    for n in AUTOTEST_NO:
        if CURVAS.match(n):
            rojo.append('R3 el patron da por curva a %s, que no lo es' % n)
    if reglas('zk-ssl-cli', *leer_arbol(ARBOL_SI)[:2]):
        rojo.append('R3 R2 y R4 rechazan el transporte declarado: su autotest falla')
    for i, (raiz, texto) in enumerate(ARBOL_NO):
        if not reglas(raiz, *leer_arbol(texto)[:2]):
            rojo.append('R3 la regla de aristas no ve el arbol sembrado %d: la puerta esta ciega' % (i + 1))
    for f in FUENTE_SI:
        if not usos_tls(f):
            rojo.append('R3 la busqueda de R5 no ve %r: esta ciega' % f)
    for f in FUENTE_NO:
        if usos_tls(f):
            rojo.append('R3 la busqueda de R5 ve %r, que no es un uso' % f)
    falta = FALTA.search(FALTA_SI[0])
    if not falta or falta.group(1) != FALTA_SI[1] or any(FALTA.search(n) for n in FALTA_NO):
        rojo.append('R3 R1 no reconoce el fallo de la cache de cargo: su autotest falla')
    total, locales, cache = set(), {}, False
    for raiz in RAICES:
        texto, error = arbol(raiz)
        if texto is None:
            falta = FALTA.search(error)
            cache = cache or bool(falta)
            causa = (', y no es una curva: a la cache de cargo le falta el fuente de `%s`' % falta.group(1) if falta
                     else ' (%s)' % (error.strip().splitlines() or ['cargo tree fallo'])[-1])
            rojo.append('R1 %s: no se pudo leer su clausura%s' % (raiz, causa))
            continue
        crates, aristas, rutas = leer_arbol(texto)
        if len(crates) < MINIMO:
            rojo.append('R3 %s: %d crates en su clausura, menos de %d: instrumento ciego' % (raiz, len(crates), MINIMO))
        total.update(crates)
        rojo.extend(reglas(raiz, crates, aristas))
        raiz_real = os.path.realpath(RAIZ) + os.sep
        locales.update({c: p for c, p in rutas.items() if os.path.realpath(p).startswith(raiz_real)})
    fuentes = []
    for c in sorted(locales):
        for d, _, ficheros in os.walk(os.path.join(locales[c], 'src')):
            fuentes.extend(os.path.join(d, f) for f in sorted(ficheros) if f.endswith('.rs'))
    for f in fuentes:
        with open(f, encoding='utf-8') as h:
            for linea, uso in usos_tls(h.read()):
                rojo.append('R5 %s:%d usa %r: el TLS es del transporte de `ureq`, no del codigo de la casa (§838)'
                            % (os.path.relpath(f, RAIZ), linea, uso))
    if len(fuentes) < MINIMO_FUENTES:
        rojo.append('R3 R5 leyo %d fuentes, menos de %d: instrumento ciego' % (len(fuentes), MINIMO_FUENTES))
    print('check_curvas: %d raices de produccion (%s), %d crates distintos en su clausura normal; '
          '%d fuentes de %d crates de la casa'
          % (len(RAICES), ', '.join(RAICES), len(total), len(fuentes), len(locales)))
    if rojo:
        for r in rojo:
            print('ROJO %s' % r)
        if cache:
            print('check_curvas: R1 lee `cargo tree --offline --all-features --target all`, que pide el fuente de '
                  'cada crate de la clausura en todos los targets, y un `cargo build` solo baja los de esta '
                  'maquina. Remedio: `cargo fetch --locked`, una vez y con red; despues, la herramienta corre sin red')
        print('check_curvas: ROJO · %d fallo(s)' % len(rojo))
        return 1
    print('check_curvas: ninguna curva del patron en la clausura de produccion fuera del transporte declarado '
          '(rustls sobre ring, por ureq, en el cli y el SDK: §838), y ningun uso de ring, rustls ni webpki en '
          'las fuentes de la casa')
    return 0


if __name__ == '__main__':
    sys.exit(main())
