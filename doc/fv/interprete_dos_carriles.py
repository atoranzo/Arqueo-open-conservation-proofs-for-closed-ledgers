#!/usr/bin/env python3
"""Intérprete generalizado — dos carriles, índices crudos, selector booleano.

Evolución de interprete_selectores.py (§185, un carril) al primer circuito
de dos carriles: circuit_frozen_climb. Tres piezas nuevas:
  1. bucle de carril `for (lane, offset) in [(0,0),(1,LANE_B)]`
  2. índices crudos `result[24+i]`, `result[44]` junto a simbólicos
  3. selector booleano `result[44] = current[COL_BIT]*(...)` — clase 'todas'
El criterio de éxito es el MISMO de §185: cazar el mutante.
"""
import os, re, sys

RAIZ = os.path.join(os.path.dirname(__file__), "..", "..", "crates", "stark-experiment", "src")
STATE_WIDTH, NUM_ROUNDS, CYCLE_LENGTH = 12, 7, 8
FROZEN_DEPTH = 24  # externa, como en el guardián

RE_CONST = re.compile(r"^(?:pub )?const ([A-Z][A-Z_0-9]*): usize = (.+?);", re.MULTILINE)

def resolver(texto):
    crudos = {m.group(1): m.group(2).strip() for m in RE_CONST.finditer(texto)}
    val = {"STATE_WIDTH":STATE_WIDTH,"NUM_ROUNDS":NUM_ROUNDS,"CYCLE_LENGTH":CYCLE_LENGTH,
           "FROZEN_DEPTH":FROZEN_DEPTH}
    for _ in range(len(crudos)+3):
        for n,e in crudos.items():
            if n in val: continue
            ee=e
            for k,v in val.items(): ee=re.sub(rf"\b{k}\b",str(v),ee)
            if re.fullmatch(r"[0-9+\-*/() ]+",ee):
                try: val[n]=eval(ee)
                except ZeroDivisionError: pass
    return val

def ev(expr,val,i=None,lane=None,offset=None):
    e=expr.strip(); s=dict(val)
    if i is not None: s["i"]=i
    if lane is not None: s["lane"]=lane
    if offset is not None: s["offset"]=offset
    for k,v in sorted(s.items(),key=lambda kv:-len(kv[0])): e=re.sub(rf"\b{re.escape(k)}\b",str(v),e)
    if re.fullmatch(r"[0-9+\-*/() ]+",e):
        try: return eval(e)
        except (ZeroDivisionError,SyntaxError): return None
    return None

# selector → clase de fila. hash_flag: 0..NUM_ROUNDS-1; link_flag: fila enlace;
# COL_BIT (booleano): TODAS las filas.
def clase_de(selector):
    if selector=="hash_flag": return "hash"
    if selector=="link_flag": return "enlace"
    return "todas"  # current[COL_BIT]*... u otros sin selector de fila

RE_RESULT = re.compile(r"result\[([^\]]+)\]\s*=\s*(.+)")

def cobertura(texto,val):
    """{(col, clase)} cubiertas por restricciones, expandiendo lane e i."""
    cub=set()
    # ALIASES: `let X = next[EXPR];` — X carga una columna. Se cosechan para
    # seguir la columna cuando X aparece en el cuerpo de una restricción.
    # Sin esto, una columna leída vía variable intermedia parece huérfana
    # (§137: censar TODAS las representaciones; §42.5: no condenar lo que no
    # se entiende — aquí SÍ se entiende, solo hay que seguir el alias).
    aliases = {}  # nombre -> expr de columna
    for am in re.finditer(r"let\s+([a-z_]+)\s*=\s*next\[([^\]]+)\]\s*;", texto):
        aliases[am.group(1)] = am.group(2)
    # ¿hay bucle de carril? detecta `for (lane, offset) in [(...)]`
    tiene_carril = "lane, offset" in texto or "lane * STATE_WIDTH" in texto
    for m in RE_RESULT.finditer(texto):
        idx_expr, cuerpo_ini = m.group(1), m.group(2)
        fin = texto.find(";", m.end()); cuerpo = texto[m.start():fin] if fin>0 else texto[m.start():m.start()+200]
        # selector: primer identificador tras '='
        sm = re.match(r"\s*([a-z_]+)\s*\*", cuerpo_ini)
        selector = sm.group(1) if sm else None
        # ¿es booleano de COL_BIT? (current[COL_BIT]*(current[COL_BIT]-...))
        if selector is None or "COL_BIT" in cuerpo_ini:
            selector = "__bool__"
        clase = clase_de(selector)
        # columnas que gobierna via next[...], expandiendo lane si aplica
        lanes = [(0,0),(1,val.get("LANE_B",STATE_WIDTH))] if tiene_carril and ("lane" in idx_expr or "offset" in cuerpo) else [(None,None)]
        for (lane,offset) in lanes:
            exprs = [nm.group(1) for nm in re.finditer(r"next\[([^\]]+)\]", cuerpo)]
            for alias_name, alias_expr in aliases.items():
                if re.search(rf"\b{alias_name}\b", cuerpo):
                    exprs.append(alias_expr)
            for ex in exprs:
                for i in range(STATE_WIDTH):
                    v = ev(ex,val,i=i,lane=lane,offset=offset)
                    if v is not None and 0<=v<val.get("TRACE_WIDTH",STATE_WIDTH):
                        cub.add((v,clase))
    return cub

def censar(texto):
    val=resolver(texto); w=val.get("TRACE_WIDTH",STATE_WIDTH)
    cub=cobertura(texto,val)
    # aserciones: (col, clase-de-su-fila)
    asev=set()
    for m in re.finditer(r"Assertion::single\(\s*([^,]+?)\s*,\s*([^,]+?)\s*,",texto):
        fila=ev(m.group(2),val)
        if fila is None: continue
        clase = "hash" if (fila%CYCLE_LENGTH)<NUM_ROUNDS else "enlace"
        for i in range(STATE_WIDTH):
            c=ev(m.group(1),val,i=i)
            if c is not None and 0<=c<w: asev.add((c,clase))
        c0=ev(m.group(1),val)
        if c0 is not None and 0<=c0<w: asev.add((c0,clase))
    # clase 'todas' cubre ambas hash/enlace
    cub_expand=set(cub)
    for (c,cl) in list(cub):
        if cl=="todas": cub_expand |= {(c,"hash"),(c,"enlace")}
    sin=[]
    for clase in ("hash","enlace"):
        for c in range(w):
            if (c,clase) in cub_expand: continue
            if (c,clase) in asev: continue
            sin.append((c,clase))
    return w,sin

def main():
    texto=open(os.path.join(RAIZ,"circuit_frozen_climb.rs"),encoding="utf-8").read()
    w,sin=censar(texto)
    print(f"== intérprete generalizado: circuit_frozen_climb, {w} columnas ==")
    print(f"   celdas sin dueño (sano): {len(sin)} {'✅' if not sin else sin[:8]}")
    # MUTANTE: borrar C_CAP del carril A (result[24 + i] = link_flag * next[i])
    mut = re.sub(r"\n\s*result\[24 \+ i\][^\n]*","",texto)
    _,sin_mut=censar(mut)
    print(f"\n== MUTANTE (result[24+i] borrado: capacidad carril A en enlace) ==")
    print(f"   celdas sin dueño: {len(sin_mut)} {sin_mut[:8] if sin_mut else '(ninguna)'}")
    if len(sin_mut)>len(sin):
        nuevas=set(sin_mut)-set(sin)
        print(f"   ✅ CAZA EL MUTANTE — nuevas huérfanas: {sorted(nuevas)}")
        return 0
    print(f"   ❌ no lo caza (sano={len(sin)}, mutante={len(sin_mut)})")
    return 1

sys.exit(main())
