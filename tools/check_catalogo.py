#!/usr/bin/env python3
# ── EL CATALOGO DE RECHAZOS (§777) ──────────────────────────────────
# Ata cada texto de rechazo que el kit (crates/zk-ssl-verify, el binario
# zk-ssl-verify) puede emitir a una entrada de spec/PAQUETE.md seccion 5, y
# cada entrada de la seccion 5 a un productor que la emite. Es el «censo por
# llamada» que el §397 hizo a mano al sellar (50 llamadas, 49 textos), y que
# desde entonces nadie re-derivo: la cabecera de main.rs:20-22 y PAQUETE.md:
# 742-745 lo prometen, y ninguna compuerta lo corria.
#
# Reglas, declaradas ANTES de mirar:
#
#  R1  CENSO POR LLAMADA, no por linea. Un productor es un literal de cadena
#      del codigo de produccion de main.rs (fuera de todo item #[cfg(test)])
#      que es el PRIMER argumento de format!, o va seguido de .into(),
#      .to_string() o .to_owned(), y que tiene por ancestro una llamada a
#      err, Err, map_err, ok_or_else u ok_or. El literal se reconstruye con
#      sus continuaciones `\` (el salto y el blanco que sigue no son texto).
#      println!/eprintln!/print! (el VERDE, las lineas n/m, el uso) quedan
#      fuera por construccion: no son format!. expect/unreachable!/panic!
#      tampoco: son invariantes (exit 101), no rechazos, y se CUENTAN.
#      ⚠️ §711: en nativo ya no salen con 101: la red del mando los recoge,
#      y salen con `el verificador entro en panico: {m}` y 1, que si esta
#      en §5.
#  R2  HUECOS DERIVADOS. Un hueco cuyo argumento es VersionCabeza::texto*()
#      vale el conjunto que DERIVA crates/zk-ssl-verify/src/lib.rs (TODAS y
#      los lleva_*); uno cuyo argumento es una const con valor literal vale
#      ese valor. En §5 se escribe con ese valor o como hueco {..}: un valor
#      DISTINTO es DIFIERE. El codigo manda.
#  R3  IDA. Cada texto del kit esta en §5, en posicion de entrada, con sus
#      huecos como {..}, con su valor derivado (R2) o con un RELLENO que el
#      propio codigo le da: el literal que la llamada pasa a ese parametro,
#      o el `let <hueco> = "..."|format!("...")` de su funcion.
#  R4  VUELTA. Cada entrada de §5 la explica un productor del kit (R3), un
#      compuesto envoltorio + texto de su biblioteca, o -en una lista
#      «con `{e}` uno de»- un texto de la biblioteca de su envoltorio.
#  R5  BIBLIOTECAS. Todo envoltorio -productor dentro de map_err(|e| ..)
#      cuyo hueco es `e`- tiene fila en TABLA_E, y toda fila tiene
#      envoltorio (en las dos direcciones, como R5 de check_dominios). Una
#      fila `enumera` exige que cada texto de su fuente este en §5; `opaco`
#      y `externo` se IMPRIMEN como exclusion: una exclusion muda es un
#      agujero.
#  R6  PRUEBA DE VIDA. Menos de MINIMOS llamadas, partidos o entradas es
#      ROJO: un censo vacio no es un hallazgo, es un instrumento ciego.
#
# Ceguera declarada: no sigue el flujo de datos (un texto armado en una
# variable y devuelto despues, fuera de los patrones de R1, no se ve); no
# deriva rellenos que no sean literales; y de una biblioteca solo ve lo que
# su fila de TABLA_E le dice que mire.
import difflib, os, re, sys

RAIZ = os.environ.get('CANON_RAIZ') or os.path.normpath(
    os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
KIT = os.path.join('crates', 'zk-ssl-verify', 'src', 'main.rs')
VERS = os.path.join('crates', 'zk-ssl-verify', 'src', 'lib.rs')
PAQ = os.path.join('spec', 'PAQUETE.md')
CONSTS_EN = ('zk-ssl-verify', 'zk-ssl-hash', 'zk-ssl-air')
MARCAS = {'err', 'Err', 'map_err', 'ok_or_else', 'ok_or'}
MINIMOS = {'llamadas': 200, 'partidos': 30, 'entradas': 150}
PARECIDO = 0.85

H = 'crates/zk-ssl-hash/src/lib.rs'
V = 'crates/zk-ssl-verify/src/'
A = 'crates/zk-ssl-air/src/'
M = 'crates/zk-ssl-medio/src/'
# fn_del_envoltorio::esqueleto -> (modo, [(fichero, selector)])  selector: fn:NOMBRE | Display:TIPO
TABLA_E = {
    'correr::no se puede leer {}: {}': ('externo', 'std::io'),
    'correr::JSON ilegible: {}': ('externo', 'serde_json'),
    'digest_de::{}: {}': ('opaco', [(H, 'Display:FormatoError')]),
    'u64_de::{}: {}': ('opaco', [(H, 'Display:HexError'), (H, 'Display:FormatoError')]),
    'verificar_paquete::cabeza: {}': ('opaco', [(V + 'lib.rs', 'Display:VerificaError')]),
    'verificar_paquete::sibling {}: {}': ('opaco', [(H, 'Display:FormatoError')]),
    'acuse_contra_cabeza::acuse: {}': ('opaco', 'Debug de zk_ssl_verify::inclusion::InclusionError'),
    'verificar_cofirmas_del_paquete::cofirma {}: {}': ('opaco', [(V + 'lib.rs', 'Display:VerificaError')]),
    'cabeza_v3_verificada::{}: cabeza: {}': ('opaco', [(V + 'lib.rs', 'Display:VerificaError')]),
    'indice_embebido::{}: {}': ('opaco', [(V + 'lib.rs', 'Display:VerificaError')]),
    'misma_continuidad::las cabezas llevan claves DISTINTAS y las actas no las unen: {}':
        ('enumera', [(V + 'actas.rs', 'Display:RotacionError')]),
    'actas_del_sobre::actas[{}]: {}': ('opaco', [(V + 'actas.rs', 'fn:acta_de_json')]),
    'camino_mmr::camino[{}]: {}': ('opaco', [(H, 'Display:FormatoError')]),
    'camino_de::{}: siblings[{}]: {}': ('opaco', [(H, 'Display:FormatoError')]),
    'verificar_rechazo::banda: {}': ('enumera', [(A + 'banda.rs', 'fn:verificar'),
                                                 (A + 'banda.rs', 'fn:comprobar_enunciado'),
                                                 (A + 'lib.rs', 'fn:comprobar_marca')]),
    'verificar_edad::edad: {}': ('enumera', [(A + 'lib.rs', 'fn:verificar_contra_cabeza'),
                                             (A + 'lib.rs', 'fn:comprobar_enunciado'),
                                             (A + 'lib.rs', 'fn:comprobar_emisor'),   # §718: la cota del emisor, que comprobar_enunciado llama
                                             (A + 'lib.rs', 'fn:verificar'),
                                             (A + 'lib.rs', 'fn:comprobar_marca')]),
    'verificar_cobro_pendiente::cobro: {}': ('enumera', [(A + 'cobro_pendiente.rs', 'fn:verificar_contra_cabeza'),
                                                         (A + 'cobro_pendiente.rs', 'fn:comprobar_enunciado'),
                                                         (A + 'cobro_pendiente.rs', 'fn:verificar'),
                                                         (A + 'lib.rs', 'fn:comprobar_marca')]),
    'verificar_pago_en_curso::pago: {}': ('enumera', [(A + 'pago_en_curso.rs', 'fn:verificar_contra_cabeza'),
                                                      (A + 'pago_en_curso.rs', 'fn:comprobar_enunciado'),
                                                      (A + 'pago_en_curso.rs', 'fn:verificar'),
                                                      (A + 'lib.rs', 'fn:comprobar_marca')]),
    'verificar_prenda::prenda: {}': ('enumera', [(A + 'prenda.rs', 'fn:verificar_contra_cabeza'),
                                                 (A + 'prenda.rs', 'fn:verificar'),
                                                 (A + 'lib.rs', 'fn:comprobar_marca')]),
    'operacion_nombrada::{}: {}: el rechazo de un lote nombra su operacion (§611)': ('kit', 'u64_de'),
    'resolver_lote::resolucion.data.campos: {}': ('kit', 'u64_de'),
    'bifurcacion_del_ancla::rama: {}': ('kit', 'u64_de'),   # §714: el formatVersion de la rama
    'resolver_prenda::resolucion.sobre: {}': ('kit', 'enunciado_de_prenda'),
    'verificar_ancla::cabeza: {}': ('opaco', [(V + 'lib.rs', 'Display:VerificaError')]),
    'verificar_ancla::contraria: {}': ('opaco', [(V + 'lib.rs', 'Display:VerificaError')]),
    'verificar_solapamiento::el acta {} de la cadena no vale: {}': ('opaco', [(V + 'actas.rs', 'Display:ActaError')]),
    'sha256_de::{}: {}': ('kit', 'hex_a_bytes'),
    'verificar_ancla_cofirmada::testigos: {}': ('opaco', [(M + 'nota.rs', 'Display:ErrorDeNota')]),
    'verificar_ancla_cofirmada::cabeza: {}': ('opaco', [(V + 'lib.rs', 'Display:VerificaError')]),
    'verificar_ancla_cofirmada::publicador: {}': ('opaco', [(M + 'nota.rs', 'Display:ErrorDeNota')]),
    'verificar_ancla_cofirmada::la nota: {}': ('opaco', [(M + 'nota.rs', 'Display:ErrorDeNota')]),
    'verificar_ancla_cofirmada::la cofirma de {} no verifica: {}': ('opaco', [(M + 'nota.rs', 'Display:ErrorDeNota')]),
    'verificar_ancla_cofirmada::el ancla de esta cabeza no esta en la posicion {} de las {} de la nota: {}':
        ('opaco', [(M + 'medio.rs', 'Display:ErrorDelMedio')]),
}
EXTERNOS_EN_LISTAS = ('winter-verifier', 'winterfell')   # `{e:?}` de winterfell: hueco puro

# ── el lexer: lo justo de Rust ──────────────────────────────────────
ESC = {'n': '\n', 't': '\t', 'r': '\r', '0': '\0', '\\': '\\', '"': '"', "'": "'"}

def lex(src):
    toks, i, n, ln = [], 0, len(src), 1
    while i < n:
        c = src[i]
        if c == '\n':
            ln += 1; i += 1; continue
        if c.isspace():
            i += 1; continue
        if src.startswith('//', i):
            j = src.find('\n', i); i = n if j < 0 else j; continue
        if src.startswith('/*', i):
            d, j = 1, i + 2
            while j < n and d:
                if src.startswith('/*', j): d += 1; j += 2
                elif src.startswith('*/', j): d -= 1; j += 2
                else:
                    ln += src[j] == '\n'; j += 1
            i = j; continue
        m = re.match(r'b?r(#*)"', src[i:i + 40])
        if m and not (i and (src[i - 1].isalnum() or src[i - 1] == '_')):
            fin = src.index('"' + m.group(1), i + m.end())
            val = src[i + m.end():fin]
            toks.append(('str', val, ln, ln + val.count('\n')))
            ln += val.count('\n'); i = fin + 1 + len(m.group(1)); continue
        if c == '"' or (c == 'b' and src.startswith('b"', i) and not (i and (src[i - 1].isalnum() or src[i - 1] == '_'))):
            j, l0, out = i + (2 if c == 'b' else 1), ln, []
            while src[j] != '"':
                if src[j] == '\\':
                    nx = src[j + 1]
                    if nx == '\n':                      # continuacion: ni el salto ni el blanco
                        j += 1
                        while src[j] in ' \t\r\n':
                            ln += src[j] == '\n'; j += 1
                        continue
                    if nx in ESC: out.append(ESC[nx]); j += 2; continue
                    if nx == 'x': out.append(chr(int(src[j + 2:j + 4], 16))); j += 4; continue
                    if nx == 'u':
                        k = src.index('}', j); out.append(chr(int(src[j + 3:k], 16))); j = k + 1; continue
                    raise ValueError('escape \\%s en la linea %d' % (nx, ln))
                ln += src[j] == '\n'; out.append(src[j]); j += 1
            toks.append(('bstr' if c == 'b' else 'str', ''.join(out), l0, ln))
            i = j + 1; continue
        if c == "'":
            if src[i + 1] == '\\':
                k = src.index("'", i + 2); toks.append(('char', src[i:k + 1], ln, ln)); i = k + 1; continue
            if i + 2 < n and src[i + 2] == "'":
                toks.append(('char', src[i:i + 3], ln, ln)); i += 3; continue
            m = re.match(r"'[A-Za-z_]\w*", src[i:]); toks.append(('vida', m.group(0), ln, ln)); i += m.end(); continue
        m = re.match(r'[A-Za-z_]\w*|\d[\w.]*|::|=>|->|.', src[i:], re.S)
        t = 'id' if (c.isalpha() or c == '_') else ('num' if c.isdigit() else 'p')
        toks.append((t, m.group(0), ln, ln)); i += m.end()
    return toks

# ── R1: el censo de un fichero ─────────────────────────────────────
CIERRA = {')': '(', ']': '[', '}': '{'}

def recorrer(toks):
    """Literales con su contexto: cadena de cabezas de '(' abiertas, fn, cierre
    de #[cfg(test)], y los cuerpos de fn e impl Display."""
    lits, pila, fns, cuerpos, displays, saltados = [], [], [], {}, {}, 0
    pend, pend_d, impl_disp = None, None, None
    i, n = 0, len(toks)
    while i < n:
        t, v = toks[i][0], toks[i][1]
        if v == '#' and [x[1] for x in toks[i + 1:i + 7]] == ['[', 'cfg', '(', 'test', ')', ']']:
            j, d = i + 7, 0                         # salta el item entero
            while j < n:
                if toks[j][1] == '{': d += 1
                elif toks[j][1] == '}':
                    d -= 1
                    if d == 0: break
                elif toks[j][1] == ';' and d == 0: break
                j += 1
            saltados += 1; i = j + 1; continue
        if v == 'fn' and i + 1 < n and toks[i + 1][0] == 'id':
            pend, pend_d = toks[i + 1][1], len(pila)
        if v == 'impl':                              # impl ... Display for TIPO {
            j = i
            while j < n and toks[j][1] != '{': j += 1
            cab = [x[1] for x in toks[i:j]]
            if 'Display' in cab and 'for' in cab:
                impl_disp = (cab[cab.index('for') + 1], len(pila))
        if v in '([{' and t == 'p':
            cab = None
            if i and toks[i - 1][0] == 'id': cab = toks[i - 1][1]
            elif i > 1 and toks[i - 1][1] == '!' and toks[i - 2][0] == 'id': cab = toks[i - 2][1] + '!'
            pila.append((v, cab, i))
            if v == '{' and pend and len(pila) - 1 == pend_d:
                fns.append((pend, len(pila))); cuerpos[pend] = [i, None]; pend = None
            if v == '{' and impl_disp and len(pila) - 1 == impl_disp[1]:
                displays[impl_disp[0]] = [i, None]; impl_disp = (impl_disp[0], -1, len(pila))
        elif v == ';' and pend and len(pila) == pend_d:
            pend = None
        elif v in ')]}' and t == 'p':
            if pila and pila[-1][0] == CIERRA[v]:
                if v == '}' and fns and fns[-1][1] == len(pila):
                    cuerpos[fns[-1][0]][1] = i; fns.pop()
                if v == '}' and impl_disp and len(impl_disp) == 3 and impl_disp[2] == len(pila):
                    displays[impl_disp[0]][1] = i; impl_disp = None
                pila.pop()
        elif t == 'str':
            cabs = [p[1] for p in pila if p[0] == '(']
            lits.append({'i': i, 'linea': toks[i][2], 'hasta': toks[i][3], 'texto': v,
                         'cabs': cabs, 'primero': bool(pila) and pila[-1][2] + 1 == i,
                         'sig': ''.join(x[1] for x in toks[i + 1:i + 4]),
                         'fn': fns[-1][0] if fns else None,
                         'pila': [p[2] for p in pila]})
        i += 1
    return lits, cuerpos, displays, saltados

def partes_de(fmt):
    """format! -> [('lit', s) | ('hueco', nombre, spec)]"""
    out, buf, i = [], [], 0
    while i < len(fmt):
        if fmt.startswith('{{', i) or fmt.startswith('}}', i):
            buf.append(fmt[i]); i += 2; continue
        if fmt[i] == '{':
            j = fmt.index('}', i)
            nombre, _, spec = fmt[i + 1:j].partition(':')
            if buf: out.append(('lit', ''.join(buf))); buf = []
            out.append(('hueco', nombre, spec)); i = j + 1; continue
        buf.append(fmt[i]); i += 1
    if buf: out.append(('lit', ''.join(buf)))
    return out

def args_tras(toks, i):
    args, cur, d, j = [], [], 0, i + 1
    while j < len(toks):
        v = toks[j][1] if toks[j][0] == 'p' else None
        if v and v in '([{': d += 1
        elif v and v in ')]}':
            if d == 0: break
            d -= 1
        if v == ',' and d == 0:
            args.append(cur); cur = []
        else:
            cur.append(toks[j])
        j += 1
    if cur: args.append(cur)
    return args

def productores(rel, toks, lits, en_rango=None, marcas=MARCAS, con_write=False):
    out = []
    for d in lits:
        if en_rango and not any(a <= d['i'] <= b for a, b in en_rango):
            continue
        cab = d['cabs'][-1] if d['cabs'] else None
        es_w = con_write and cab in ('write!', 'writeln!')
        es_f = cab == 'format!' and d['primero']
        es_i = d['sig'] in ('.into(', '.to_string(', '.to_owned(') or d['sig'].startswith('.into')
        if not (es_f or es_i or es_w):
            continue
        anc = [c for c in (d['cabs'][:-1] if (es_f or es_w) else d['cabs']) if c in marcas]
        if not (anc or es_w):
            continue
        args = [' '.join(x[1] for x in a).replace(' ', '') for a in args_tras(toks, d['i'])][1:] if (es_f or es_w) else []
        partes, pos = [], 0
        for p in (partes_de(d['texto']) if (es_f or es_w) else [('lit', d['texto'])]):
            if p[0] == 'lit':
                partes.append(p); continue
            nombre, spec = p[1], p[2]
            if nombre == '' or nombre.isdigit():
                arg = args[pos] if pos < len(args) else '?'; pos += 1
            else:
                arg = next((a.split('=', 1)[1] for a in args if a.startswith(nombre + '=')), nombre)
            partes.append(('hueco', nombre, spec, arg))
        mapa = None
        if 'map_err' in d['cabs']:
            k = d['i']
            while not (toks[k][1] == 'map_err' and toks[k + 1][1] == '('): k -= 1
            if toks[k + 2][1] == '|':
                j = k + 3
                while toks[j][1] != '|': j += 1
                mapa = {x[1] for x in toks[k + 3:j] if x[0] == 'id'}
        out.append({'fichero': rel, 'linea': d['linea'], 'partido': d['hasta'] > d['linea'],
                    'fn': d['fn'], 'partes': partes, 'via': 'err' if 'err' in anc else (anc[0] if anc else 'write!'),
                    'param': mapa, 'i': d['i']})
    return out

# ── R2: lo derivable ────────────────────────────────────────────────
def conjuntos_de_version(src):
    """texto*() de VersionCabeza, derivados como los deriva lib.rs (texto_de)."""
    m = re.search(r'const TODAS:\s*\[VersionCabeza;\s*\d+\]\s*=\s*\[([^\]]*)\]', src)
    disc = dict(re.findall(r'\b(V\d+)\s*=\s*(\d+),', src))
    todas = [x.strip().split('::')[-1] for x in m.group(1).split(',') if x.strip()]
    pred = {}
    for nombre, neg, cuerpo in re.findall(r'pub fn (lleva_\w+)\(self\) -> bool \{\s*(!?)matches!\(self, ([^)]*)\)', src):
        pred[nombre] = ({x.strip().split('::')[-1] for x in cuerpo.split('|')}, neg == '!')
    def texto_de(vs):
        vs = ['v%s' % disc[v] for v in vs]
        return vs[-1] if len(vs) == 1 else '%s o %s' % (', '.join(vs[:-1]), vs[-1])
    out = {'texto': texto_de(todas)}
    for fn, p in re.findall(r'pub fn (texto_con_\w+)\(\) -> String \{\s*Self::texto_de\(Self::TODAS\.iter\(\)\.copied\(\)\.filter\(\|v\| v\.(lleva_\w+)\(\)\)\)', src):
        out[fn] = texto_de([v for v in todas if (v in pred[p][0]) != pred[p][1]])
    return out

def constantes():
    out = {}
    for crate in CONSTS_EN:
        base = os.path.join(RAIZ, 'crates', crate, 'src')
        for dp, _, fs in os.walk(base):
            for f in fs:
                if not f.endswith('.rs'): continue
                for nombre, valor in re.findall(r'^\s*(?:pub(?:\([^)]*\))?\s+)?const ([A-Z][A-Z0-9_]*):\s*[^=]+=\s*("(?:[^"\\]|\\.)*"|0x[0-9a-fA-F_]+|\d[\d_]*)\s*;',
                                                open(os.path.join(dp, f), encoding='utf-8').read(), re.M):
                    v = valor[1:-1] if valor.startswith('"') else (int(valor.replace('_', ''), 0))
                    out.setdefault(nombre, set()).add(v)
    return {k: next(iter(v)) for k, v in out.items() if len(v) == 1}   # ambigua: queda hueco

# ── el catalogo: §5 ─────────────────────────────────────────────────
def norm(t):
    t = re.sub(r'\s+', ' ', t.strip()).replace('0x…', '{}').replace('…', '{}')
    return re.sub(r'\{[^{}]*\}', '{}', t)

def spans_de_entrada(s, pos=0):
    """Desde s[pos]=='`': los spans separados por '` · `', hasta el primer cierre con
    un numero PAR de '`' dentro (asi viven los textos con `v` o `tipo` dentro)."""
    segs = []
    while pos < len(s) and s[pos] == '`':
        pos += 1
        cierres = sorted([(m.start(), m.group(0)) for m in re.finditer(r'`(?: · `| — |, | \(|\.|$| y | o |: |;)', s) if m.start() >= pos])
        ok = [c for c in cierres if s[pos:c[0]].count('`') % 2 == 0]
        if not ok: break
        j, pat = ok[0]
        segs.append((s[pos:j], j))
        if pat == '` · `':
            pos = j + 4; continue
        break
    return segs

def variantes(s):
    """`A[ B]`: B es opcional (un segundo write! del mismo brazo, actas.rs:329); `X …`
    en una lista es un prefijo («forma de traza …»)."""
    m = re.search(r'\[([ ,][^\]]*)\]', s)
    base, frags = (s[:m.start()] + s[m.end():], [norm(m.group(1))]) if m else (s, [])
    return {'n': norm(base), 'frags': frags, 'prefijo': s.rstrip().endswith('…')}

LISTA = re.compile(r'(?:uno de(?: estos)?(?:,[^:`]*)?|es el rojo de `[^`]+`):\s*$|(?:uno de(?: estos)?|es el rojo de `[^`]+`):\s*')

def entradas(rel):
    lineas = open(os.path.join(RAIZ, rel), encoding='utf-8').read().split('\n')
    ini = next(k for k, l in enumerate(lineas) if l.startswith('## 5.'))
    fin = next(k for k, l in enumerate(lineas) if k > ini and l.startswith('## '))
    items, bloque = [], None
    for k in range(ini + 1, fin):
        l = lineas[k]
        m = re.match(r'^\*\*(.+?)\*\*', l)
        if m: bloque = m.group(1); continue
        m = re.match(r'^( *)- (.*)$', l)
        if m: items.append([k + 1, len(m.group(1)) // 2, bloque, m.group(2)]); continue
        if items and l.startswith('  ') and l.strip() and items[-1][0] is not None:
            items[-1][3] += ' ' + l.strip(); continue
        if not l.strip() or not l.startswith(' '): items.append([None, 0, bloque, ''])
    out, padre = [], {}
    for n, nivel, bloque, t in items:
        if n is None: padre = {}; continue
        lead = re.match(r'^[^`]{0,40}?: (?=`)', t) if nivel else None
        p0 = lead.end() if lead else 0
        segs = spans_de_entrada(t, p0) if t[p0:p0 + 1] == '`' else []
        env = padre.get(nivel - 1) if nivel else None
        for s, _ in segs:
            out.append(dict(variantes(s), linea=n, bloque=bloque, texto=s,
                            clase='lista' if env else 'entrada', envoltorio=env))
        # una lista «con `{e}` uno de: ...» en la misma viñeta
        cola = t[segs[-1][1]:] if segs else ''
        m = LISTA.search(cola)
        if segs and m and m.end() < len(cola):
            for s in re.findall(r'`([^`]+)`', cola[m.end():]):
                if ' ' in s or '{' in s:
                    out.append(dict(variantes(s), linea=n, bloque=bloque, texto=s,
                                    clase='lista', envoltorio=norm(segs[-1][0])))
        if segs:
            padre = {k: v for k, v in padre.items() if k < nivel}
            padre[nivel] = norm(segs[-1][0]) if re.search(r'uno de(?: estos)?[^`]*:\s*$', t) else padre.get(nivel - 1)
    return out

# ── R3/R4: el cruce ─────────────────────────────────────────────────
def main():
    rojo, avisos = [], []
    src_kit = open(os.path.join(RAIZ, KIT), encoding='utf-8').read()
    toks = lex(src_kit)
    lits, cuerpos, _, saltados = recorrer(toks)
    ps = productores(KIT, toks, lits)
    vers = conjuntos_de_version(open(os.path.join(RAIZ, VERS), encoding='utf-8').read())
    consts = constantes()
    invariantes = sum(1 for d in lits if d['cabs'] and d['cabs'][-1] in ('expect', 'unreachable!', 'panic!'))

    # firmas: fn -> parametros, para los rellenos de R3
    firmas = {}
    for i, t in enumerate(toks):
        if t[1] == 'fn' and toks[i + 1][0] == 'id':
            j = i + 2
            while toks[j][1] != '(': j += 1
            d, k, ps_ = 0, j, []
            while True:
                v = toks[k][1] if toks[k][0] in ('p', 'id') else ''
                if v and v in '([{': d += 1
                elif v and v in ')]}': d -= 1
                elif d == 1 and toks[k][0] == 'id' and toks[k + 1][1] == ':' and toks[k - 1][1] in '(,':
                    ps_.append(v)
                k += 1
                if d == 0: break
            firmas[toks[i + 1][1]] = ps_

    def rellenos(fn, hueco):
        out = set()
        if fn in firmas and hueco in firmas[fn]:
            idx = firmas[fn].index(hueco)
            for i, t in enumerate(toks):
                if t[1] == fn and i + 1 < len(toks) and toks[i + 1][1] == '(' and toks[i - 1][1] != 'fn':
                    a = args_tras(toks, i + 1)
                    if idx < len(a):
                        s = [x for x in a[idx] if x[0] == 'str']
                        if s and (len(a[idx]) == 1 or any(x[1] == 'format' for x in a[idx])):
                            out.add(s[0][1])
        if fn in cuerpos and cuerpos[fn][1]:
            a, b = cuerpos[fn]
            for i in range(a, b):
                if toks[i][1] == 'let' and toks[i + 1][1] == hueco and toks[i + 2][1] == '=':
                    if toks[i + 3][0] == 'str': out.add(toks[i + 3][1])
                    elif toks[i + 3][1] == 'format' and toks[i + 6][0] == 'str': out.add(toks[i + 6][1])
        return out

    def valor_derivado(arg):
        m = re.search(r'VersionCabeza::(texto\w*)\(\)$', arg)
        if m:
            if m.group(1) not in vers:
                rojo.append('R2 %s: VersionCabeza::%s() no se deriva de %s' % (KIT, m.group(1), VERS))
                return None
            return vers[m.group(1)]
        base = arg.split('::')[-1]
        return consts.get(base) if re.fullmatch(r'[A-Z][A-Z0-9_]*', base) else None

    def plano(p):
        s = []
        for x in p['partes']:
            if x[0] == 'lit': s.append(x[1]); continue
            v = valor_derivado(x[3])
            s.append(str(v) if v is not None else '{}')
        return norm(''.join(s))

    def regex(p, lib=None):
        r = []
        for x in p['partes']:
            if x[0] == 'lit':
                r.append(re.escape(re.sub(r'\s+', ' ', x[1]))); continue
            _, nombre, spec, arg = x
            alts = [r'\{\}']
            v = valor_derivado(arg)
            if v is not None: alts.append(re.escape(str(v)))
            if nombre and not nombre.isdigit():
                alts += [re.escape(norm(f)) for f in rellenos(p['fn'], nombre)]
            if lib and p['param'] and nombre in p['param']:
                alts += lib
            r.append('(?:%s)' % '|'.join(sorted(set(alts))))
        return re.compile(''.join(r) + r'\Z')

    cat = entradas(PAQ)
    if len(ps) < MINIMOS['llamadas'] or sum(p['partido'] for p in ps) < MINIMOS['partidos'] or len(cat) < MINIMOS['entradas']:
        rojo.append('R6 censo ciego: %d llamadas, %d partidas, %d entradas (minimos %s)' % (
            len(ps), sum(p['partido'] for p in ps), len(cat), MINIMOS))

    # R5: envoltorios y su biblioteca
    libtextos = {}
    envs = {}
    for p in ps:
        if p['param'] and any(x[0] == 'hueco' and x[1] in p['param'] for x in p['partes']):
            clave = '%s::%s' % (p['fn'], plano(p))
            envs.setdefault(clave, []).append(p)
    for clave in sorted(set(envs) - set(TABLA_E)):
        rojo.append('R5 %s:%d envoltorio sin fila en TABLA_E: %s' % (KIT, envs[clave][0]['linea'], clave))
    for clave in sorted(set(TABLA_E) - set(envs)):
        rojo.append('R5 fila de TABLA_E sin envoltorio en %s: %s' % (KIT, clave))
    cache = {}
    for clave, (modo, fuentes) in sorted(TABLA_E.items()):
        if clave not in envs: continue
        if not isinstance(fuentes, list):
            avisos.append('excluido (%s) %s -> %s' % (modo, clave, fuentes)); continue
        textos = []
        for f, sel in fuentes:
            if f not in cache:
                tk = lex(open(os.path.join(RAIZ, f), encoding='utf-8').read())
                cache[f] = (tk,) + recorrer(tk)
            tk, ls, cu, di, _ = cache[f]
            tipo, nombre = sel.split(':')
            rango = cu.get(nombre) if tipo == 'fn' else di.get(nombre)
            if not rango or rango[1] is None:
                rojo.append('R5 %s: no encuentro %s (fila %s)' % (f, sel, clave)); continue
            lp = productores(f, tk, ls, [tuple(rango)], marcas=MARCAS, con_write=(tipo == 'Display'))
            # `let q = "..."; return Err(q.into())` (zk-ssl-air/src/lib.rs:735)
            for i in range(rango[0], rango[1]):
                if tk[i][1] == 'let' and tk[i + 2][1] == '=' and tk[i + 3][0] == 'str':
                    lp.append({'fichero': f, 'linea': tk[i + 3][2], 'partes': [('lit', tk[i + 3][1])], 'fn': nombre, 'param': None})
            textos += lp
        libtextos[clave] = (modo, textos)
        if modo != 'enumera':
            avisos.append('excluido (opaco) %s: %d texto(s) de %s, sin enumerar en §5' % (
                clave, len(textos), ', '.join(sorted({f for f, _ in fuentes}))))

    def cubre(t, e):
        plano_t = norm(''.join(x[1] if x[0] == 'lit' else '{}' for x in t['partes']))
        if re.fullmatch(rx_lib(t), e['n']) or plano_t in e['frags']:
            return True
        return e['prefijo'] and plano_t.startswith(e['n'][:-2])

    def rx_lib(t):
        r = []
        for x in t['partes']:
            if x[0] == 'lit': r.append(re.escape(re.sub(r'\s+', ' ', x[1])))
            else:
                v = valor_derivado(x[3]) if len(x) > 3 else None
                r.append('(?:\\{\\}%s)' % ('|' + re.escape(str(v)) if v is not None else ''))
        return ''.join(r)

    # R3: ida
    explicadas, faltan = set(), []
    rxs = []
    for p in ps:
        rx = regex(p)
        lib = []
        clave = '%s::%s' % (p['fn'], plano(p))
        if clave in libtextos:
            lib = [rx_lib(t) for t in libtextos[clave][1]]
        rx_c = regex(p, lib) if lib else None
        rxs.append((p, rx, rx_c))
        hit = [k for k, e in enumerate(cat) if rx.match(e['n'])]
        explicadas.update(hit)
        if not hit: faltan.append(p)
    # R4: vuelta (compuestos y listas de biblioteca)
    for k, e in enumerate(cat):
        if k in explicadas: continue
        if any(rc and rc.match(e['n']) for _, _, rc in rxs):
            explicadas.add(k); continue
        if e['clase'] == 'lista':
            for clave, (modo, textos) in libtextos.items():
                if clave.split('::', 1)[1] == e['envoltorio'] and any(cubre(t, e) for t in textos):
                    explicadas.add(k); break
            if e['n'] == '{}' and k not in explicadas:
                explicadas.add(k)               # `{e:?}`, el de winterfell
    for clave, (modo, textos) in libtextos.items():
        if modo != 'enumera': continue
        for t in textos:
            if not any(cubre(t, e) for e in cat):
                faltan.append(dict(t, lib=clave))
    sobran = [e for k, e in enumerate(cat) if k not in explicadas]

    # DIFIERE: uno a uno, el mas parecido
    textos = {}
    for p in faltan:
        textos.setdefault(plano(p) if 'lib' not in p else norm(''.join(x[1] if x[0] == 'lit' else '{}' for x in p['partes'])), []).append(p)
    libres = {}
    for e in sobran:
        libres.setdefault(e['n'], []).append(e)
    for t, ps_t in sorted(textos.items(), key=lambda kv: (kv[1][0]['fichero'], kv[1][0]['linea'])):
        sitio = '%s:%s' % (ps_t[0]['fichero'], ','.join(str(p['linea']) for p in ps_t))
        mejor = max(libres, key=lambda n: difflib.SequenceMatcher(None, t, n).ratio(), default=None)
        if mejor and difflib.SequenceMatcher(None, t, mejor).ratio() >= PARECIDO:
            es = libres.pop(mejor)
            rojo.append('R3 DIFIERE %s el kit dice «%s» y %s:%s dice «%s»' % (
                sitio, t, PAQ, ','.join(str(e['linea']) for e in es), es[0]['texto']))
        else:
            rojo.append('R3 FALTA %s «%s»%s' % (sitio, t, (' (biblioteca de %s)' % ps_t[0]['lib']) if 'lib' in ps_t[0] else ''))
    for es in libres.values():
        for e in es:
            rojo.append('R4 SOBRA %s:%d [%s] «%s»' % (PAQ, e['linea'], e['bloque'], e['texto']))

    distintos = len({plano(p) for p in ps})
    print('check_catalogo: %s: %d llamadas de rechazo, %d textos distintos (%d partidas con \\ en el fuente)'
          % (KIT, len(ps), distintos, sum(p['partido'] for p in ps)))
    print('check_catalogo: %s seccion 5: %d entradas (%d de listas de biblioteca)'
          % (PAQ, len(cat), sum(e['clase'] == 'lista' for e in cat)))
    print('check_catalogo: conjuntos de VersionCabeza (%s): %s' % (VERS, ' · '.join('%s()=%s' % kv for kv in sorted(vers.items()))))
    print('  excluidos por construccion: println!/eprintln! (no son format!), %d invariante(s) expect/unreachable!, %d item(s) #[cfg(test)]'
          % (invariantes, saltados))
    for a in avisos: print('  %s' % a)
    if rojo:
        for r in rojo: print('ROJO %s' % r)
        print('check_catalogo: ROJO · %d fallo(s)' % len(rojo))
        return 1
    print('check_catalogo: los %d textos del kit estan en §5 y las %d entradas de §5 salen del kit o de su biblioteca'
          % (distintos, len(cat)))
    return 0

if __name__ == '__main__':
    sys.exit(main())
