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
las dos se derivan del MISMO byte: PUBLICADA_PAGO_B. Lo que NO vale es
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

Ceguera declarada:
  - no ve una cifra escrita con palabras;
  - no entra en AUDITORIA.md: los asientos son REGISTRO HISTORICO y una
    cifra vieja dentro de un asiento es correcta;
  - no entra en doc/preprints/: entrada 28, SUSPENDIDA hasta el fin del
    proyecto. Lo que alli quede rancio se DECLARA, no se repara;
  - no vigila TIEMPOS ni RATIOS: dependen de la maquina y publicarlos en
    absoluto es una decision de diseno, no un numero que atar. Declarado
    en el asiento del §304.
"""
import os
import re
import sys

RAIZ = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FUENTE = os.path.join(RAIZ, "crates", "zk-ssl", "src", "metrics.rs")
VENTANA = 2

EXCLUIDOS = {
    "AUDITORIA.md": "registro historico: una cifra vieja en un asiento es correcta",
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
CIFRA = re.compile(r"(\d{2,3})[.,](\d)\s*(MiB|MB)\b")


def constante():
    """Lee la fuente unica. Por ESTRUCTURA, no por posicion."""
    try:
        with open(FUENTE, encoding="utf-8") as fh:
            src = fh.read()
    except OSError as exc:
        print("ROJO: no se puede leer la fuente unica: %s" % exc)
        sys.exit(1)
    m = re.search(r"const PUBLICADA_PAGO_B:\s*usize\s*=\s*([0-9_]+)", src)
    if not m:
        print("ROJO: no aparece PUBLICADA_PAGO_B en %s" % FUENTE)
        print("      la fuente unica es esa constante: sin ella no hay atado")
        sys.exit(1)
    b = int(m.group(1).replace("_", ""))
    f = re.search(r'const PUBLICADA_FECHA:\s*&str\s*=\s*"([^"]+)"', src)
    mib = "%.1f" % (b * 1000 / 1048576.0)
    msi = "%.1f" % (b * 1000 / 1000000.0)
    return b, mib, msi, (f.group(1) if f else "sin fecha")


def documentos():
    for nombre in sorted(os.listdir(RAIZ)):
        if nombre.endswith(".md") and nombre not in EXCLUIDOS:
            yield nombre


def main():
    pago_b, mib, msi, fecha = constante()
    par_mib = tuple(mib.split("."))
    par_msi = tuple(msi.split("."))

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
                if HISTORICA.search(bloque):
                    saltadas.append(
                        (nombre, i + 1,
                         "%s,%s %s - cita de una version anterior"
                         % (par[0], par[1], unidad))
                    )
                    continue
                vistos += 1
                if par == par_mib and unidad == "MiB":
                    continue
                if par == par_msi and unidad == "MB":
                    continue
                if par == par_mib and unidad == "MB":
                    fallos.append(
                        (nombre, i + 1, "UNIDAD",
                         "el valor binario lleva etiqueta MB; o MiB, o el "
                         "equivalente SI %s MB" % msi)
                    )
                else:
                    fallos.append(
                        (nombre, i + 1, "VALOR",
                         "dice %s,%s %s; se esperan %s MiB o %s MB"
                         % (par[0], par[1], unidad, mib, msi))
                    )

    print("check_publicadas: %d B por pago -> %s MiB / %s MB por mil (medido el %s)"
          % (pago_b, mib, msi, fecha))
    print("  %d citas examinadas en documentos del repo" % vistos)
    for nombre, razon in sorted(EXCLUIDOS.items()):
        print("  excluido %s - %s" % (nombre, razon))
    print("  excluido doc/preprints/ - entrada 28, SUSPENDIDA hasta el fin del proyecto")
    for nombre, n, detalle in saltadas:
        print("  SALTADA  %-18s :%-5d %s" % (nombre, n, detalle))

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

    print("OK: todas las citas cuadran en valor y en unidad")
    return 0


if __name__ == "__main__":
    sys.exit(main())
