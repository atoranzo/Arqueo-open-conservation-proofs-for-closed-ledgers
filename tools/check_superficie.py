#!/usr/bin/env python3
# ── LA SUPERFICIE, POR SU PRODUCTOR (§801, RFC-0021 E3a) ──────────────
# spec/RPC.md §«Versionado y compatibilidad» es la promesa del protocolo (RFC-0021 E2): seis
# superficies estables, cada una con su identificador, y lo que las rompe. Una promesa que nada
# compara con lo que el arbol produce es texto (RFC-0021, «Seguridad»), y la 90 pide «una compuerta
# que la haga cierta: lo declarado estable no cambia sin que el canon lo diga». Esta herramienta lee
# cada superficie de su PRODUCTOR y la compara con la LISTA que la seccion fija para la version
# vigente, en su bloque ```superficie```. Las cifras viven en la lista, no en la prosa (D-C). Reglas,
# declaradas ANTES de mirar; son las siete de la D-D del RFC-0021:
#
#  R1  LA VERSION DEL CABLE. Sus escrituras de produccion -cada literal de cadena "zkssl/0.N" de
#      crates/*/src fuera de todo item #[cfg(test)]- son las que la lista declara en
#      `cable.escrituras` (hoy no tiene UN productor: hecho 5 y R7 del RFC), y todas dicen la de
#      `cable`; la dicen tambien spec/openrpc.json (info.version y el `const` de ProtocolVersion),
#      la respuesta del nodo a zkssl_protocolVersion, el titulo de spec/RPC.md y su fila de
#      zkssl_protocolVersion. Y el kit es 0.N.x con el cable 0.N (R3 de la D-B).
#  R2  LOS METODOS. Cada `metodo` de la lista esta en spec/openrpc.json con sus parametros -nombre,
#      esquema y si es opcional- y su resultado: uno que falta, que cambia de nombre o de tipo, o un
#      parametro que pasa a obligatorio es ROJO; un zkssl_* nuevo, o un parametro opcional nuevo,
#      se CUENTA y no es ROJO (R2 de la D-B). Los dev_* quedan fuera de la promesa (D-A). Cada
#      `esquema` de la lista -los que el OpenRPC da con forma: la codificacion- no cambia. Y
#      spec/RPC.md, fuera de esta seccion, nombra exactamente los metodos de spec/openrpc.json.
#  R3  LO FIRMADO. VersionCabeza::TODAS (crates/zk-ssl-verify/src/lib.rs) solo crece frente a
#      `cabeza.acepta`, y VERSION_FORMATO es miembro e igual a `cabeza.emite`: una version de
#      cabeza nueva que el nodo emite rompe el cable (decision 3 del §799). Igual la cofirma
#      (COFIRMA_VERSION es `cofirma.emite`; COFIRMA_V_MAX no baja de `cofirma.lee-hasta`) y el
#      acta (ACTA_VERSION es `acta.emite`).
#  R4  LOS SOBRES. Los `tipo` del despacho del kit (verificar_paquete, en
#      crates/zk-ssl-verify/src/main.rs) y los `v` que lee solo crecen frente a `sobre.tipo` y
#      `sobre.v`, y frente a spec/PAQUETE.md §2: un `tipo` o un `v` que la seccion 2 escribe y el
#      kit no lee es ROJO.
#  R5  EL CONTRATO DEL KIT. Los codigos de salida -los de codigo_de_salida y el del uso en main-
#      son los de `kit.salida`, y son los que dicen spec/PAQUETE.md §6 (su vineta de los codigos) y
#      la seccion 0 de doc/KIT.md y de doc/KIT_EN.md (el parrafo del contrato, hasta su primera
#      nota ⚠️): el hecho 6 del RFC, el manual contaba tres. Un codigo, en esos textos, es un digito
#      suelto, con comillas invertidas o sin ellas.
#  R6  LOS VECTORES. No repite tools/check_vectores.py: exige que tools/canon.sh lo corra. Cada
#      `manifiesto` de la lista existe en spec/vectors/; los .json y las lineas de manifiesto solo
#      crecen frente a `vectores.json` y `vectores.lineas`; cada `familia` sigue en FAMILIAS de
#      tools/artefacto.sh, el productor del artefacto del kit. Lo nuevo se cuenta.
#  R7  PRUEBA DE VIDA, en los dos sentidos (como la del §787). En cada pasada, cada comparacion se
#      repite con la lista y una entrada que el productor no da, con la lista vacia y con el
#      productor sin una de las entradas de la lista: las tres tienen que salir ROJO. Una clave de
#      la lista que falta, que se repite o que la herramienta no conoce, y una seccion sin bloque,
#      son ROJO.
#
# Ceguera declarada. Lo aditivo se cuenta y no es ROJO (R2 de la D-B), asi que la lista no se pone
# al dia sola: un metodo que entra sin que la lista lo diga y sale despues, dentro de la misma
# version, no se ve. Del OpenRPC mira nombres, esquemas con forma y referencias: la forma de un tipo
# con nombre (Params, EpochHead...) la escribe spec/RPC.md y la atan los vectores del cable, como los
# errores y su causa en `data`. No ve las pruebas que viajan (la superficie 2): es la E3b, con el
# corte. Del contrato del kit mira los codigos, no las lineas de VERDE. Una version escrita con
# format! o partida en dos literales no se ve, y un fichero de tests entero, declarado con
# `#[cfg(test)] mod x;` en su padre, se lee como produccion (hoy ninguno escribe la version). Los
# lectores de R1, R3, R4 y R5 son expresiones sobre el fuente de Rust SIN sus comentarios (el
# lexer los quita: una variante, un brazo o un `=> 3` comentado no cuentan), pero no lo compilan:
# una constante o un brazo de los que la lista fija, escritos de otra forma que la de hoy, no se
# leen, y su comparacion sale ROJO, no verde. En los textos de R5, un digito suelto que no es un
# codigo -un «(1)»- cuenta como uno. Del kit publicado -el tag, el commit y las huellas- no ve nada:
# no hay red.
#
# Uso: python3 tools/check_superficie.py           # exit 0 si nada se rompe; 1 con cada ROJO
#      python3 tools/check_superficie.py --lista   # imprime la lista que dan HOY los productores,
#                                                  # para fijarla en un corte; no escribe nada
import json, os, re, sys
from collections import Counter

RAIZ = os.environ.get('CANON_RAIZ') or os.path.normpath(
    os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
RPC = os.path.join('spec', 'RPC.md')
OPENRPC = os.path.join('spec', 'openrpc.json')
PAQ = os.path.join('spec', 'PAQUETE.md')
KITS = (os.path.join('doc', 'KIT.md'), os.path.join('doc', 'KIT_EN.md'))
VERS = os.path.join('crates', 'zk-ssl-verify', 'src', 'lib.rs')
ACTAS = os.path.join('crates', 'zk-ssl-verify', 'src', 'actas.rs')
KIT = os.path.join('crates', 'zk-ssl-verify', 'src', 'main.rs')
KIT_TOML = os.path.join('crates', 'zk-ssl-verify', 'Cargo.toml')
NODO = os.path.join('crates', 'zk-ssl-node', 'src', 'main.rs')
CANON = os.path.join('tools', 'canon.sh')
ARTEFACTO = os.path.join('tools', 'artefacto.sh')
VECTORES = os.path.join('spec', 'vectors')
SECCION = 'Versionado y compatibilidad'
CLAVES = ('cable', 'cable.escrituras', 'metodo', 'esquema', 'cabeza.acepta', 'cabeza.emite',
          'cofirma.emite', 'cofirma.lee-hasta', 'acta.emite', 'sobre.v', 'sobre.tipo',
          'kit.salida', 'vectores.json', 'vectores.lineas', 'manifiesto', 'familia')
REPETIBLES = ('metodo', 'esquema', 'manifiesto')
FANTASMA = 'fantasma-de-la-prueba-de-vida'


def leer(rel):
    with open(os.path.join(RAIZ, rel), encoding='utf-8') as fh:
        return fh.read()


# ── el lexer: lo justo de Rust para separar codigo, cadenas y comentarios ──
def lex(src, comentarios=None):
    """Da (clase, valor, linea): 'str' con su contenido, 'id', o 'p' (un signo). Si se le pasa una
    lista en `comentarios`, anota en ella el tramo [i, j) de cada comentario."""
    toks, i, n, ln = [], 0, len(src), 1
    while i < n:
        c = src[i]
        if c == '\n':
            ln += 1; i += 1; continue
        if c.isspace():
            i += 1; continue
        if src.startswith('//', i):
            j = src.find('\n', i); j = n if j < 0 else j
            if comentarios is not None:
                comentarios.append((i, j))
            i = j; continue
        if src.startswith('/*', i):
            d, j = 1, i + 2
            while j < n and d:
                if src.startswith('/*', j): d += 1; j += 2
                elif src.startswith('*/', j): d -= 1; j += 2
                else:
                    ln += src[j] == '\n'; j += 1
            if comentarios is not None:
                comentarios.append((i, j))
            i = j; continue
        m = re.match(r'b?r(#*)"', src[i:i + 40])
        if m and not (i and (src[i - 1].isalnum() or src[i - 1] == '_')):
            fin = src.index('"' + m.group(1), i + m.end())
            val = src[i + m.end():fin]
            toks.append(('str', val, ln)); ln += val.count('\n')
            i = fin + 1 + len(m.group(1)); continue
        if c == '"' or (c == 'b' and src.startswith('b"', i)):
            j = i + (2 if c == 'b' else 1); val = []
            while src[j] != '"':
                if src[j] == '\\':
                    val.append(src[j:j + 2]); j += 2
                else:
                    val.append(src[j]); j += 1
            val = ''.join(val)
            toks.append(('str', val, ln)); ln += val.count('\n'); i = j + 1; continue
        if c == "'":
            m = re.match(r"'(\\.[^']*|[^'\\])'", src[i:i + 12])
            if m:
                i += m.end(); continue          # un char: no es un signo
            i += 1; continue                    # un lifetime
        if c.isalpha() or c == '_':
            m = re.match(r'\w+', src[i:i + 200])
            toks.append(('id', m.group(0), ln)); i += m.end(); continue
        toks.append(('p', c, ln)); i += 1
    return toks


def sin_comentarios(src):
    """El fuente con cada comentario -tambien los `///` y los `/* */`- cambiado por blancos, con sus
    saltos de linea en su sitio: las expresiones de los lectores de R1, R3 y R4/R5 miran codigo, no
    lo que un comentario dice que es codigo, y las lineas siguen siendo las del fichero."""
    tramos, trozos, prev = [], [], 0
    lex(src, tramos)
    for i, j in tramos:
        trozos.append(src[prev:i])
        trozos.append(re.sub(r'[^\n]', ' ', src[i:j]))
        prev = j
    trozos.append(src[prev:])
    return ''.join(trozos)


def sin_tests(toks):
    """Los tokens fuera de todo item que lleva #[cfg(test)] (el atributo exacto): el item acaba en
    su `;` sin abrir nada, o en la llave que cierra su cuerpo."""
    fuera, i, n = [], 0, len(toks)
    while i < n:
        v = [t[1] for t in toks[i:i + 7]]
        if v == ['#', '[', 'cfg', '(', 'test', ')', ']'] and all(t[0] != 'str'
                                                               for t in toks[i:i + 7]):
            j, prof = i + 7, 0
            while j < n:
                clase, s = toks[j][0], toks[j][1]
                if clase == 'p' and s in '([{':
                    prof += 1
                elif clase == 'p' and s in ')]}':
                    prof -= 1
                    if prof == 0 and s == '}':
                        break
                elif clase == 'p' and s == ';' and prof == 0:
                    break
                j += 1
            i = j + 1; continue
        fuera.append(toks[i]); i += 1
    return fuera


def escrituras_del_cable():
    """R1: cada literal "zkssl/0.N" de produccion -> [(ruta bajo crates/, version, linea)]."""
    out = []
    base = os.path.join(RAIZ, 'crates')
    for crate in sorted(os.listdir(base)):
        src = os.path.join(base, crate, 'src')
        for raiz, _, fs in os.walk(src):
            for f in sorted(fs):
                if not f.endswith('.rs'):
                    continue
                ruta = os.path.join(raiz, f)
                rel = os.path.relpath(ruta, base).replace(os.sep, '/')
                texto = open(ruta, encoding='utf-8').read()
                if 'zkssl/0.' not in texto:
                    continue
                for clase, val, ln in sin_tests(lex(texto)):
                    if clase == 'str' and re.fullmatch(r'zkssl/0\.\d+', val):
                        out.append((rel, val, ln))
    return out


# ── los productores ──────────────────────────────────────────────────
def corto(esquema):
    ref = esquema.get('$ref')
    if ref:
        return ref.rsplit('/', 1)[-1]
    return json.dumps(esquema, sort_keys=True, separators=(',', ':'))


def firma(m):
    ps = ['%s%s: %s' % (p['name'], '' if p.get('required', False) else '?', corto(p['schema']))
          for p in m['params']]
    return '%s(%s) -> %s' % (m['name'], ', '.join(ps), corto(m['result']['schema']))


def esquema_con_forma(nombre, s):
    linea = '%s %s' % (nombre, s['type'])
    for k in ('pattern', 'const'):
        if k in s:
            linea += ' %s %s' % (k, s[k])
    return linea


def productores():
    """Cada superficie leida de su productor, como texto, para la lista y para comparar."""
    p = {}
    orpc = json.loads(leer(OPENRPC))
    p['openrpc'] = orpc
    p['metodo'] = [firma(m) for m in orpc['methods'] if m['name'].startswith('zkssl_')]
    p['dev'] = [m['name'] for m in orpc['methods'] if not m['name'].startswith('zkssl_')]
    p['esquema'] = [esquema_con_forma(k, s) for k, s in sorted(orpc['components']['schemas'].items())
                    if 'type' in s]
    esc = escrituras_del_cable()
    p['escrituras'] = esc
    cuenta = Counter(rel for rel, _, _ in esc)
    p['cable.escrituras'] = [r if k == 1 else '%s×%d' % (r, k) for r, k in sorted(cuenta.items())]
    p['cable'] = sorted({v for _, v, _ in esc})
    lib = sin_comentarios(leer(VERS))
    enum = lib.find('pub enum VersionCabeza')
    variantes = dict(re.findall(r'\n\s*(V\d+)\s*=\s*(\d+),', lib[enum:])) if enum >= 0 else {}
    m = re.search(r'pub const TODAS: \[VersionCabeza; \d+\] = \[(.*?)\];', lib, re.S)
    todas = re.findall(r'VersionCabeza::(V\d+)', m.group(1)) if m else []
    p['cabeza.acepta'] = ['v' + variantes[v] for v in todas if v in variantes]
    m = re.search(r'pub const VERSION_FORMATO: u8 = (\d+);', lib)
    p['cabeza.emite'] = ['v' + m.group(1)] if m else []
    m = re.search(r'pub const COFIRMA_VERSION: u64 = (\d+);', lib)
    p['cofirma.emite'] = [m.group(1)] if m else []
    m = re.search(r'pub const COFIRMA_V_MAX: u64 = (\d+);', lib)
    p['cofirma.lee-hasta'] = [m.group(1)] if m else []
    m = re.search(r'pub const ACTA_VERSION: u8 = (\d+);', sin_comentarios(leer(ACTAS)))
    p['acta.emite'] = [m.group(1)] if m else []
    kit = sin_comentarios(leer(KIT))
    cuerpo = funcion(kit, 'verificar_paquete')
    p['sobre.v'] = sorted(set(re.findall(r'v_paquete != (\d+)', cuerpo)), key=int)
    p['sobre.tipo'] = re.findall(r'Some\("([a-z_-]+)"\) => return verificar_', cuerpo)
    codigos = set(re.findall(r'=> (\d+),', funcion(kit, 'codigo_de_salida')))
    codigos |= set(re.findall(r'ExitCode::from\((\d+)\)', funcion(kit, 'main')))
    p['kit.salida'] = sorted(codigos, key=int)
    vec = os.path.join(RAIZ, VECTORES)
    jsons, manifiestos, lineas = 0, [], 0
    for raiz, _, fs in os.walk(vec):
        for f in fs:
            if f.endswith('.json'):
                jsons += 1
            if f == 'MANIFIESTO.txt':
                rel = os.path.relpath(os.path.join(raiz, f), vec).replace(os.sep, '/')
                manifiestos.append(rel)
                lineas += len(open(os.path.join(raiz, f), encoding='utf-8').read().splitlines())
    p['vectores.json'] = [str(jsons)]
    p['vectores.lineas'] = [str(lineas)]
    p['manifiesto'] = sorted(manifiestos)
    m = re.search(r'^FAMILIAS="([^"]*)"', leer(ARTEFACTO), re.M)
    p['familia'] = m.group(1).split() if m else []
    return p


def funcion(src, nombre):
    """Las lineas de `fn nombre`, de su firma a la llave que cierra su cuerpo (con el lexer: una
    llave dentro de una cadena o de un comentario no cuenta). Vacio si no esta."""
    toks = lex(src)
    for k in range(len(toks) - 1):
        if toks[k][:2] == ('id', 'fn') and toks[k + 1][:2] == ('id', nombre):
            j, prof = k + 2, 0
            while j < len(toks) and not (toks[j][0] == 'p' and toks[j][1] == '{'):
                j += 1
            for t in toks[j:]:
                if t[0] == 'p' and t[1] == '{':
                    prof += 1
                elif t[0] == 'p' and t[1] == '}':
                    prof -= 1
                    if prof == 0:
                        return '\n'.join(src.splitlines()[toks[k][2] - 1:t[2]])
            return ''
    return ''


# ── la lista de la seccion ───────────────────────────────────────────
def la_lista():
    """(lista {clave: [valores]}, rojos, texto de RPC.md antes de la seccion)."""
    rpc = leer(RPC)
    m = re.search(r'^## %s\s*$' % re.escape(SECCION), rpc, re.M)
    if not m:
        return {}, ['R7 %s no tiene la seccion «%s»: no hay lista que comparar' % (RPC, SECCION)], rpc
    antes, resto = rpc[:m.start()], rpc[m.end():]
    sig = re.search(r'^## ', resto, re.M)
    seccion = resto[:sig.start()] if sig else resto
    b = re.search(r'^```superficie\n(.*?)^```', seccion, re.M | re.S)
    if not b:
        return {}, ['R7 la seccion «%s» no tiene su bloque ```superficie```' % SECCION], antes
    lista, rojos = {}, []
    for linea in b.group(1).splitlines():
        if not linea.strip() or linea.lstrip().startswith('#'):
            continue
        clave, _, valor = linea.strip().partition(' ')
        if clave not in CLAVES:
            rojos.append('R7 la lista lleva una clave que no conozco: %s' % clave)
            continue
        if clave in lista and clave not in REPETIBLES:
            rojos.append('R7 la lista repite la clave %s' % clave)
        if clave in REPETIBLES:
            lista.setdefault(clave, []).append(valor.strip())
        else:
            lista[clave] = valor.split()
    for clave in CLAVES:
        if not lista.get(clave):
            rojos.append('R7 la lista no dice %s, o lo dice vacio: instrumento ciego' % clave)
    return lista, rojos, antes


# ── comparar ─────────────────────────────────────────────────────────
def parse_firma(s):
    m = re.fullmatch(r'(\w+)\((.*)\) -> (\S+)', s.strip())
    if not m:
        return None
    ps = {}
    for p in filter(None, (x.strip() for x in m.group(2).split(', '))):
        q = re.fullmatch(r'(\w+)(\??): (\S+)', p)
        if not q:
            return None
        ps[q.group(1)] = (q.group(2) == '?', q.group(3))
    return m.group(1), ps, m.group(3)


def comparar(modo, fijado, producido, quien=('la lista', 'el productor')):
    """-> (rojos, nuevos), listas de textos. Una lista vacia es ROJO en todo modo. `quien` nombra
    los dos lados en el texto: lo fijado y lo producido."""
    if not fijado:
        return ['%s no dice nada: instrumento ciego' % quien[0]], []
    if modo == 'igual':
        a, b = set(fijado), set(producido)
        r = ['%s dice %s y %s no' % (quien[0], x, quien[1]) for x in sorted(a - b)]
        r += ['%s dice %s y %s no' % (quien[1], x, quien[0]) for x in sorted(b - a)]
        return r, []
    if modo in ('crece', 'miembro'):
        a, b = set(fijado), set(producido)
        return (['%s sale: %s lo dice y %s no' % (x, quien[0], quien[1]) for x in sorted(a - b)],
                ['%s entra' % x for x in sorted(b - a)] if modo == 'crece' else [])
    if modo == 'minimo':
        f, p = int(fijado[0]), int(producido[0]) if producido else -1
        if p < f:
            return ['la lista fija %d y hay %d: solo crece' % (f, p)], []
        return [], (['%d mas que los %d de la lista' % (p - f, f)] if p > f else [])
    if modo == 'firmas':
        prod = {x.split('(', 1)[0].split(' ', 1)[0]: x for x in producido}
        r, n = [], []
        vistos = set()
        for s in fijado:
            pf = parse_firma(s)
            if not pf:
                r.append('la lista escribe mal %s' % s); continue
            nombre, ps, res = pf
            vistos.add(nombre)
            if nombre not in prod:
                r.append('%s: la lista lo fija y el productor ya no lo da (quitado o renombrado)' % nombre)
                continue
            pp = parse_firma(prod[nombre])
            if not pp:
                r.append('%s: no leo su firma en el productor: %s' % (nombre, prod[nombre])); continue
            _, qs, qres = pp
            if qres != res:
                r.append('%s: el resultado era %s y es %s' % (nombre, res, qres))
            for k, (opc, esq) in ps.items():
                if k not in qs:
                    r.append('%s: el parametro %s ya no esta' % (nombre, k))
                elif qs[k][1] != esq:
                    r.append('%s: el parametro %s era %s y es %s' % (nombre, k, esq, qs[k][1]))
                elif opc and not qs[k][0]:
                    r.append('%s: el parametro %s era opcional y es obligatorio' % (nombre, k))
                elif not opc and qs[k][0]:
                    n.append('%s: el parametro %s pasa a opcional' % (nombre, k))
            for k, (opc, esq) in qs.items():
                if k not in ps:
                    if opc:
                        n.append('%s: parametro opcional nuevo %s' % (nombre, k))
                    else:
                        r.append('%s: parametro OBLIGATORIO nuevo %s' % (nombre, k))
        n += ['%s entra' % x for x in sorted(set(prod) - vistos)]
        return r, n
    if modo == 'esquemas':
        prod = {x.split(' ', 1)[0]: x for x in producido}
        fij = {x.split(' ', 1)[0]: x for x in fijado}
        r = []
        for k, v in sorted(fij.items()):
            if k not in prod:
                r.append('esquema %s: la lista lo fija y el OpenRPC ya no lo da' % k)
            elif prod[k] != v:
                r.append('esquema %s: era «%s» y es «%s»' % (k, v, prod[k]))
        return r, ['esquema %s entra' % k for k in sorted(set(prod) - set(fij))]
    raise ValueError(modo)


def mutantes(modo, fijado, producido):
    """R7: (que se muto, fijado, producido) que TIENEN que salir ROJO."""
    out = [('la lista vacia', [], producido)]
    if modo == 'minimo':
        p = int(producido[0]) if producido else 0
        out.append(('la lista por encima del productor', [str(p + 1)], producido))
        if fijado:
            out.append(('el productor por debajo de la lista', fijado, [str(int(fijado[0]) - 1)]))
        return out
    if modo == 'firmas':
        out.append(('un metodo fantasma en la lista',
                    fijado + ['zkssl_%s() -> Q' % FANTASMA.replace('-', '')], producido))
    elif modo == 'esquemas':
        out.append(('un esquema fantasma en la lista', fijado + ['Fantasma string'], producido))
    else:
        out.append(('una entrada fantasma en la lista', fijado + [FANTASMA], producido))
    if fijado:
        if modo == 'firmas':
            quita = fijado[0].split('(', 1)[0]
            out.append(('el productor sin %s' % quita, fijado,
                        [x for x in producido if not x.startswith(quita + '(')]))
        elif modo == 'esquemas':
            quita = fijado[0].split(' ', 1)[0]
            out.append(('el productor sin %s' % quita, fijado,
                        [x for x in producido if x.split(' ', 1)[0] != quita]))
        else:
            out.append(('el productor sin %s' % fijado[0], fijado, [x for x in producido if x != fijado[0]]))
    return out


def main():
    if '--lista' in sys.argv[1:]:
        p = productores()
        print('```superficie')
        for clave in CLAVES:
            if clave in REPETIBLES:
                for v in p[clave]:
                    print('%s %s' % (clave, v))
            else:
                print('%s %s' % (clave, ' '.join(p[clave])))
        print('```')
        return 0

    rojo, nuevo, vida = [], [], []
    lista, r, rpc_antes = la_lista()
    rojo += r
    p = productores()
    orpc = p['openrpc']
    rpc = leer(RPC)

    # (regla, que se compara, modo, fijado, producido): todas pasan por comparar() y por R7
    cmp = []
    L = lambda k: lista.get(k, [])
    version = (L('cable') or ['?'])[0]
    # R1
    cmp.append(('R1', 'la version de las escrituras de produccion', 'igual', L('cable'), p['cable']))
    cmp.append(('R1', 'las escrituras de produccion de la version', 'igual', L('cable.escrituras'),
                p['cable.escrituras']))
    sch = orpc['components']['schemas'].get('ProtocolVersion', {})
    cmp.append(('R1', 'la version de spec/openrpc.json (info.version y ProtocolVersion)', 'igual',
                L('cable'), sorted({orpc['info'].get('version', ''), sch.get('const', '')} - {''})))
    nodo = re.findall(r'"zkssl_protocolVersion" => Ok\(json!\("(zkssl/0\.\d+)"\)\)',
                      sin_comentarios(leer(NODO)))
    cmp.append(('R1', 'la respuesta del nodo a zkssl_protocolVersion', 'igual', L('cable'), nodo))
    titulo = re.findall(r'\A# .*\(`(zkssl/0\.\d+)`\)', rpc)
    fila = re.findall(r'^\| `zkssl_protocolVersion` \| — \| `"(zkssl/0\.\d+)"` \|', rpc, re.M)
    cmp.append(('R1', 'el titulo de spec/RPC.md y su fila de zkssl_protocolVersion', 'igual',
                L('cable'), sorted(set(titulo) | set(fila)) if titulo and fila else []))
    m = re.search(r'^version = "(\d+)\.(\d+)\.\d+"', leer(KIT_TOML), re.M)
    cmp.append(('R1', 'el kit 0.N.x con el cable 0.N (R3)', 'igual', L('cable')[:1],
                ['zkssl/%s.%s' % m.groups()] if m else []))
    # R2
    cmp.append(('R2', 'los metodos zkssl_* de spec/openrpc.json', 'firmas', L('metodo'), p['metodo']))
    cmp.append(('R2', 'los esquemas con forma de spec/openrpc.json', 'esquemas', L('esquema'),
                p['esquema']))
    en_rpc = sorted(set(re.findall(r'\b(?:zkssl|dev)_[A-Za-z]+\b', rpc_antes)))
    cmp.append(('R2', 'los metodos que nombra spec/RPC.md, frente a los de spec/openrpc.json', 'igual',
                [m['name'] for m in orpc['methods']], en_rpc, ('spec/openrpc.json', 'spec/RPC.md')))
    # R3
    cmp.append(('R3', 'VersionCabeza::TODAS', 'crece', L('cabeza.acepta'), p['cabeza.acepta']))
    cmp.append(('R3', 'VERSION_FORMATO, la que el nodo firma', 'igual', L('cabeza.emite'),
                p['cabeza.emite']))
    cmp.append(('R3', 'VERSION_FORMATO, miembro de VersionCabeza::TODAS', 'miembro', p['cabeza.emite'],
                p['cabeza.acepta'], ('VERSION_FORMATO', 'VersionCabeza::TODAS')))
    cmp.append(('R3', 'COFIRMA_VERSION, la que el testigo estampa', 'igual', L('cofirma.emite'),
                p['cofirma.emite']))
    cmp.append(('R3', 'COFIRMA_V_MAX, la mas alta que se lee', 'minimo', L('cofirma.lee-hasta'),
                p['cofirma.lee-hasta']))
    cmp.append(('R3', 'ACTA_VERSION', 'igual', L('acta.emite'), p['acta.emite']))
    # R4
    cmp.append(('R4', 'los `v` que lee el kit', 'crece', L('sobre.v'), p['sobre.v']))
    cmp.append(('R4', 'los `tipo` del despacho del kit', 'crece', L('sobre.tipo'), p['sobre.tipo']))
    paq = leer(PAQ)
    s2 = seccion_de(paq, r'## 2\. ', r'## 3\. ')
    # la forma de arriba de cada esqueleto: `{ "v": N, "tipo": "..."` o el `"v": N,` del paquete de
    # posicion; los `tipo` de dentro (la resolucion de la completitud) y el `v` del ancla no son formas
    cmp.append(('R4', 'los `tipo` de spec/PAQUETE.md §2, frente al kit', 'miembro',
                sorted(set(re.findall(r'^\{ "v": \d+, "tipo": "([^"]+)"', s2, re.M))),
                p['sobre.tipo'], ('spec/PAQUETE.md §2', 'el kit')))
    cmp.append(('R4', 'los `v` de spec/PAQUETE.md §2, frente al kit', 'miembro',
                sorted({a or b for a, b in
                        re.findall(r'^\{ "v": (\d+), "tipo"|^  "v": (\d+),\s*$', s2, re.M)}),
                p['sobre.v'], ('spec/PAQUETE.md §2', 'el kit')))
    # R5
    cmp.append(('R5', 'los codigos de salida del kit', 'igual', L('kit.salida'), p['kit.salida']))
    s6 = seccion_de(paq, r'## 6\. ', r'### 6\.1 ')
    vi = re.search(r'^- \*\*[^*\n]*c[oó]digos de salida:\*\*.*?(?=^- |\Z)', s6, re.M | re.S)
    cmp.append(('R5', 'los codigos que dice spec/PAQUETE.md §6', 'igual', p['kit.salida'],
                codigos_en(vi.group(0)) if vi else [], ('el kit', 'spec/PAQUETE.md §6')))
    for doc, frase in zip(KITS, ('El contrato del mando', 'The command contract')):
        s0 = seccion_de(leer(doc), r'## 0\. ', r'## 1\. ')
        par = next((x for x in re.split(r'\n\s*\n', s0) if frase in x), '')
        par = par.split('⚠️', 1)[0][par.find(frase):] if par else ''
        cmp.append(('R5', 'los codigos que dice la seccion 0 de %s' % doc, 'igual', p['kit.salida'],
                    codigos_en(par), ('el kit', doc)))
    # R6
    canon = '\n'.join(x for x in leer(CANON).splitlines() if not x.lstrip().startswith('#'))
    # una llamada sin argumentos, como la de 2 quater: la de `--desde` solo cuenta sus R6
    cmp.append(('R6', 'tools/check_vectores.py, en lo que tools/canon.sh corre por su nombre', 'miembro',
                ['tools/check_vectores.py'],
                sorted(set(re.findall(r'python3 (tools/\w+\.py)\s*(?:>|$)', canon, re.M))),
                ('la regla', CANON)))
    cmp.append(('R6', 'los .json de spec/vectors/', 'minimo', L('vectores.json'), p['vectores.json']))
    cmp.append(('R6', 'las lineas de manifiesto de spec/vectors/', 'minimo', L('vectores.lineas'),
                p['vectores.lineas']))
    cmp.append(('R6', 'los manifiestos de spec/vectors/', 'crece', L('manifiesto'), p['manifiesto']))
    cmp.append(('R6', 'FAMILIAS de tools/artefacto.sh', 'crece', L('familia'), p['familia']))

    for regla, que, modo, fijado, producido, *quien in cmp:
        quien = quien[0] if quien else ('la lista', 'el productor')
        r, n = comparar(modo, fijado, producido, quien)
        rojo += ['%s %s: %s' % (regla, que, x) for x in r]
        nuevo += ['%s %s: %s' % (regla, que, x) for x in n]
        for como, f, q in mutantes(modo, fijado, producido):
            if not comparar(modo, f, q, quien)[0]:
                vida.append('R7 %s: con %s no sale ROJO: la comparacion esta ciega' % (que, como))

    print('check_superficie: %s, %d metodos zkssl_* (%d en spec/openrpc.json, %d dev_* fuera de la '
          'promesa), %d escrituras de la version; cabezas %s, emite %s; %d tipo de sobre y v %s; '
          'salidas %s; %s .json y %s lineas de %d manifiestos; %d familias del artefacto'
          % (', '.join(p['cable']) or '?', len(p['metodo']), len(orpc['methods']), len(p['dev']),
             len(p['escrituras']), ' '.join(p['cabeza.acepta']), ' '.join(p['cabeza.emite']),
             len(p['sobre.tipo']), ' '.join(p['sobre.v']), ' '.join(p['kit.salida']),
             p['vectores.json'][0], p['vectores.lineas'][0], len(p['manifiesto']), len(p['familia'])))
    for x in nuevo:
        print('NUEVO %s (aditivo, R2 de la D-B: se cuenta y no es ROJO)' % x)
    rojo += vida
    if rojo:
        for x in rojo:
            print('ROJO %s' % x)
        print('check_superficie: ROJO · %d fallo(s), en %d comparaciones' % (len(rojo), len(cmp)))
        return 1
    print('check_superficie: la superficie de %s es la de su lista en %d comparaciones, con su prueba '
          'de vida; %d novedad(es) aditiva(s) que la lista no dice' % (version, len(cmp), len(nuevo)))
    return 0


def codigos_en(texto):
    """R5: los codigos de salida que escribe un texto: cada digito suelto, entre comillas
    invertidas o sin ellas (`0`, o 0). No cuenta el que va pegado a una letra, a un punto, a un `§`
    o a otro digito: ni x86_64, ni 0.4.2, ni §573, ni §5."""
    return sorted(set(re.findall(r'(?<![\w.§])`?(\d)`?(?![\w.])', texto)), key=int)


def seccion_de(texto, desde, hasta):
    m = re.search('^' + desde, texto, re.M)
    if not m:
        return ''
    f = re.search('^' + hasta, texto[m.end():], re.M)
    return texto[m.start():m.end() + f.start()] if f else texto[m.start():]


if __name__ == '__main__':
    sys.exit(main())
