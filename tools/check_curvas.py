#!/usr/bin/env python3
# ── LAS RAICES DE PRODUCCION, SIN CURVAS (§788, ARQ-08 P1) ──────────
# La familia de supuestos de la casa es la resistencia del hash (§106): STARK/FRI, Rescue, Blake3,
# Merkle y XMSS; la unica excepcion declarada es ML-DSA-44 en el medio del ancla (§705, RFC-0013
# D-C), que no es una curva. Los comparativos del workspace -Groth16, Nova, Plonk, halo2- traen
# curvas al Cargo.lock, y hasta el §788 que no llegaran a los binarios de produccion lo sostenia la
# disciplina: un `use ark_ff` en el nodo compilaba (doc/blueprint-v2.md, ARQ-08). Reglas, declaradas
# ANTES de mirar:
#
#  R1  RAICES: los cuatro binarios y bibliotecas que un operador o un tercero ejecutan -el nodo,
#      el testigo (zk-ssl-cli), el SDK y el kit-. Su clausura es la de `cargo tree --locked
#      --offline -e normal`: lo que se enlaza, sin dependencias de desarrollo ni de compilacion.
#  R2  NINGUN crate de esa clausura casa con CURVAS: las bibliotecas de curvas elipticas y de
#      emparejamientos conocidas (arkworks, BLS12-381, BN254, Pasta, curve25519, secp256k1, P-256...).
#  R3  PRUEBA DE VIDA: cada raiz tiene al menos MINIMO crates en su clausura (una clausura vacia es
#      un instrumento ciego), y el patron reconoce los nombres de su autotest: si el patron se
#      rompe, la puerta no puede salir verde por no reconocer nada.
#
# Ceguera declarada: mira NOMBRES de crate, no codigo; una curva escrita a mano dentro de un crate
# de la casa, o en un crate con otro nombre, no se ve. Y no mira los comparativos: que sigan en el
# lock es la parte P2/P3 de ARQ-08, que no esta adoptada.
import os, re, subprocess, sys

RAIZ = os.environ.get('CANON_RAIZ') or os.path.normpath(
    os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
RAICES = ('zk-ssl-node', 'zk-ssl-cli', 'zk-ssl-sdk', 'zk-ssl-verify')
MINIMO = 50
CURVAS = re.compile(r'^(ark-.*|bls12[-_]381.*|blst|bn254.*|ark_bn254|pairing|halo2curves|pasta_curves|'
                    r'jubjub|curve25519-dalek.*|ed25519-dalek|x25519-dalek|k256|p256|p384|p521|'
                    r'secp256k1.*|libsecp256k1.*|elliptic-curve|primeorder|group|ff|ec-gpu.*)$')
AUTOTEST_SI = ('ark-ff', 'ark-bls12-381', 'bls12_381', 'curve25519-dalek', 'k256', 'secp256k1', 'pairing', 'group')
AUTOTEST_NO = ('winterfell', 'blake3', 'sha2', 'ml-dsa', 'xmss', 'sled', 'zk-ssl-hash', 'groupy-no')


def clausura(raiz):
    r = subprocess.run(['cargo', 'tree', '--locked', '--offline', '-q', '-p', raiz, '-e', 'normal',
                        '--prefix', 'none'], cwd=RAIZ, capture_output=True, text=True)
    if r.returncode != 0:
        return None, r.stderr.strip().splitlines()[-1:] or ['cargo tree fallo']
    return sorted({l.split()[0] for l in r.stdout.splitlines() if l.strip()}), None


def main():
    rojo = []
    for n in AUTOTEST_SI:
        if not CURVAS.match(n):
            rojo.append('R3 el patron no reconoce %s, que es una curva: la puerta esta ciega' % n)
    for n in AUTOTEST_NO:
        if CURVAS.match(n):
            rojo.append('R3 el patron da por curva a %s, que no lo es' % n)
    total = set()
    for raiz in RAICES:
        crates, error = clausura(raiz)
        if crates is None:
            rojo.append('R1 %s: no se pudo leer su clausura (%s)' % (raiz, error[0]))
            continue
        if len(crates) < MINIMO:
            rojo.append('R3 %s: %d crates en su clausura, menos de %d: instrumento ciego' % (raiz, len(crates), MINIMO))
        total.update(crates)
        for c in crates:
            if CURVAS.match(c):
                rojo.append('R2 %s enlaza %s, de las bibliotecas de curvas: la familia de supuestos de produccion es el hash (§106)' % (raiz, c))
    print('check_curvas: %d raices de produccion (%s), %d crates distintos en su clausura normal'
          % (len(RAICES), ', '.join(RAICES), len(total)))
    if rojo:
        for r in rojo:
            print('ROJO %s' % r)
        print('check_curvas: ROJO · %d fallo(s)' % len(rojo))
        return 1
    print('check_curvas: ninguna curva en la clausura de produccion')
    return 0


if __name__ == '__main__':
    sys.exit(main())
