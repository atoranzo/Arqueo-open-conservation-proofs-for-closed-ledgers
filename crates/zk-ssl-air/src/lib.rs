//! **zk-ssl-air: los AIR que un tercero VERIFICA, sin el probador** (RFC-0007 D-F; E4b-1, §463).
//!
//! Este crate lleva lo que hace falta para JUZGAR una prueba y nada de lo que hace falta para
//! PRODUCIRLA: el AIR (sus restricciones, sus aserciones y sus entradas publicas), el verificador
//! de `winter-verifier` y las raices nativas que el juez recompone. La traza y el `Prover` viven en
//! `stark-experiment` (`circuit_edad`), que depende de este crate: el AIR tiene UN productor, y el
//! probador y el kit lo comparten. Depende de `winter-air`, `winter-crypto`, `winter-math` y
//! `winter-verifier` sueltos y clavados con `=`, y de `zk-ssl-hash`; no arrastra `winter-prover`.
//!
//! ## La prueba de edad (RFC-0007 D-E), en una frase
//!
//! Sobre las posiciones `0..next_pending` de los arboles de pendientes y de meta, cuya raiz firma
//! la cabeza v5: el numero de posiciones VIVAS con `seq - nacido >= T` (y, si se nombra, con ese
//! emisor) es a lo sumo `K`. La caja vacia es `K = 0`; el tope por cuenta, `T = 0`; la
//! concentracion por cuenta, un emisor nombrado. Las formas por IMPORTE no se prueban: el operador
//! no guarda la apertura del compromiso (RFC-0003 D-2) y lo que el circuito no restringe no se
//! afirma (decision D-1 de E4b-1, por la constitucion).
//!
//! ## Como se cablea el arbol: un argumento de multiconjunto (decision D-5)
//!
//! La traza tiene `N = 2^m` ciclos de ocho filas (siete rondas de Rescue y una de enlace). El ciclo
//! `c` calcula el nodo `c` de los dos arboles en orden de monticulo (hijos `2c` y `2c+1`, hojas en
//! `N + p`) y la hoja de meta de la posicion `c`. Quien alimenta a quien no lo ve una restriccion
//! de dos filas: lo ata una suma acumulada en la traza AUXILIAR, `sum 1/(beta - cod(escrito)) - sum
//! 1/(beta - cod(leido)) = 1/(beta - cod(raiz A)) + 1/(beta - cod(raiz B))`, con `cod` una
//! combinacion aleatoria de etiqueta, digest y carril. Cada etiqueta se escribe y se lee una sola
//! vez, y la unica que no se lee es la raiz, que el verificador conoce.
//!
//! ## Lo que NO prueba
//!
//! Nada sobre lo que nunca entro en el arbol (eso es H5b); nada sobre importes ni saldos; nada
//! sobre otra cabeza que la que firma las dos raices. La subida de `m` a 32 niveles la hace el
//! JUEZ, en nativo ([`raiz_desde_subraiz`]), con las constantes de subarbol vacio: el AIR no
//! depende de la profundidad. Y un pendiente vivo cuya meta sea la hoja cero (legado anterior a
//! R-2a) no se puede probar: la prueba falla cerrada (decision D-2).

use winter_air::proof::Proof;
use winter_air::{
    Air, AirContext, Assertion, AuxRandElements, BatchingMethod, EvaluationFrame, FieldExtension,
    ProofOptions, TraceInfo, TransitionConstraintDegree,
};
use winter_crypto::hashers::{Blake3_256, Rp64_256};
use winter_crypto::{DefaultRandomCoin, MerkleTree};
use winter_math::fields::f64::BaseElement;
use winter_math::{ExtensionOf, FieldElement, ToElements};
use winter_verifier::{verify, AcceptableOptions};
use zk_ssl_hash::{native_merge, DOMINIO_META_PENDIENTE};

/// El digest de la casa: cuatro elementos (el de `zk-ssl-hash`).
pub type Digest = zk_ssl_hash::Digest;

type Blake3 = Blake3_256<BaseElement>;

// ------------------------------------------------------------------ la forma de la traza

/// Filas por ciclo: siete rondas de Rescue y la fila de enlace.
pub const CICLO: usize = 8;
/// El estado de Rescue (`Rp64_256`).
pub const ESTADO: usize = 12;
const RONDAS: usize = 7;
/// La profundidad de los arboles de pendientes y de meta de la capa.
pub const PROFUNDIDAD: usize = 32;
/// Bits del resto que prueba que una posicion es JOVEN (`T - 1 - edad`).
pub const BITS: usize = 32;

/// Carril A: el arbol de pendientes (doce columnas del estado).
pub const C_A: usize = 0;
/// Carril B: el arbol de meta.
pub const C_B: usize = 12;
/// Carril M: la hoja de meta, `commit_operation(PMETA_V1, [emisor, nacido])`.
pub const C_M: usize = 24;
/// La hoja del arbol de pendientes de la posicion del ciclo (cuatro columnas).
pub const C_P: usize = 36;
pub const C_EMISOR: usize = 40;
pub const C_NACIDO: usize = 41;
/// 1 si la hoja de pendientes no es cero (una posicion viva).
pub const C_VIVO: usize = 42;
/// 1 si la edad alcanza `T`; 0 exige el resto en `BITS` bits.
pub const C_QA: usize = 43;
/// 1 si el emisor casa con el nombrado; 0 exige su inverso.
pub const C_QS: usize = 44;
pub const C_WINV: usize = 45;
/// `vivo * qa * qs`: lo que la posicion suma a la cuenta.
pub const C_Z: usize = 46;
pub const C_CUENTA: usize = 47;
/// El numero de ciclo `c`: la etiqueta del nodo y de la hoja.
pub const C_CICLO: usize = 48;
/// 0 en el ciclo 0 (no hay nodo 0), 1 despues.
pub const C_ACT: usize = 49;
/// 1 si `c >= n`: por encima de la marca todo es la hoja cero.
pub const C_FUERA: usize = 50;
pub const C_BITS: usize = 51;
/// Ancho de la traza principal.
pub const ANCHO: usize = C_BITS + BITS;

/// Traza auxiliar: cinco inversos y la suma acumulada.
pub const X_H: usize = 0;
pub const X_S: usize = 5;
pub const ANCHO_AUX: usize = 6;
/// Dos aleatorios: `alfa` (la combinacion) y `beta` (el desplazamiento).
pub const ALEATORIOS: usize = 2;

const CARRIL_A: u64 = 1;
const CARRIL_B: u64 = 2;

// Columnas periodicas (longitud CICLO).
const P_HASH: usize = 0;
const P_F0: usize = 1;
const P_F6: usize = 2;
const P_ARK1: usize = 3;
const P_ARK2: usize = P_ARK1 + ESTADO;

/// Restricciones de la traza principal y de la auxiliar, contadas por familia en `new`.
pub const PRINCIPALES: usize = 3 * ESTADO + 8 + ESTADO + 4 + 5 + 1 + 1 + 1 + (4 + BITS) + 2 + 1 + 3;
pub const AUXILIARES: usize = 5 + 3 + 1;

// ------------------------------------------------------------------ las opciones

/// **Las opciones que el juez ACEPTA, y solo esas** (decision D-4). Son las `proof_options()` de
/// la capa; un test de la capa las ata (dos literales son dos productores si nadie los cruza).
pub fn opciones() -> ProofOptions {
    ProofOptions::new(
        42,
        16,
        21,
        FieldExtension::Quadratic,
        8,
        31,
        BatchingMethod::Linear,
        BatchingMethod::Linear,
    )
}

// ------------------------------------------------------------------ las entradas publicas

/// **Lo que el juez sabe.** Las dos subraices son las del subarbol `[0, 2^m)`; el juez las sube a
/// 32 niveles y las compara con `pendingRoot` y `pmetaRoot` de la cabeza v5 que firma `seq` y
/// `next_pending` (= `n`). `k` es la cota que la prueba demuestra: el probador solo puede contar DE
/// MAS, nunca esconder una posicion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EdadPublicInputs {
    pub subraiz_pend: Digest,
    pub subraiz_meta: Digest,
    pub m: u32,
    pub n: u64,
    pub seq: u64,
    pub t: u64,
    pub emisor: u64,
    pub todos: bool,
    pub k: u64,
}

impl ToElements<BaseElement> for EdadPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut v = Vec::with_capacity(15);
        v.extend_from_slice(&self.subraiz_pend);
        v.extend_from_slice(&self.subraiz_meta);
        let resto = [
            self.m as u64,
            self.n,
            self.seq,
            self.t,
            self.emisor,
            self.todos as u64,
            self.k,
        ];
        for x in resto {
            v.push(BaseElement::new(x));
        }
        v
    }
}

/// Lo que el enunciado tiene que cumplir ANTES de construir el AIR: fuera de estos rangos la
/// aritmetica del campo daria la vuelta, o el AIR no tendria forma. Nunca entra en panico.
pub fn comprobar_enunciado(pi: &EdadPublicInputs) -> Result<(), String> {
    if pi.m < 1 || pi.m > 24 {
        return Err(format!("m = {} fuera de 1..=24", pi.m));
    }
    if pi.n > (1u64 << pi.m) {
        return Err(format!("n = {} no cabe en 2^{}", pi.n, pi.m));
    }
    if pi.t >= (1u64 << BITS) || pi.seq >= (1u64 << BITS) {
        return Err(format!("T = {} o seq = {} no caben en {BITS} bits", pi.t, pi.seq));
    }
    if pi.k > pi.n {
        return Err(format!("k = {} mayor que n = {}", pi.k, pi.n));
    }
    Ok(())
}

// ------------------------------------------------------------------ la codificacion

/// `[1, alfa, alfa^2, ..., alfa^5]`.
pub fn potencias<E: FieldElement>(alfa: E) -> [E; 6] {
    let mut p = [E::ONE; 6];
    for i in 1..6 {
        p[i] = p[i - 1] * alfa;
    }
    p
}

/// **El denominador de un termino del multiconjunto**: `beta - (etiqueta + sum alfa^i v_i +
/// alfa^5 carril)`. El probador y el AIR llaman a ESTA funcion: dos codificaciones serian dos
/// productores del mismo hecho.
pub fn codificar<E: FieldElement>(pot: &[E; 6], beta: E, etiqueta: E, v: [E; 4], carril: E) -> E {
    beta - (etiqueta + pot[1] * v[0] + pot[2] * v[1] + pot[3] * v[2] + pot[4] * v[3]
        + pot[5] * carril)
}

/// El carril como elemento, para el probador.
pub fn carril<E: FieldElement<BaseField = BaseElement>>(b: bool) -> E {
    E::from(BaseElement::new(if b { CARRIL_B } else { CARRIL_A }))
}

// ------------------------------------------------------------------ la ronda de Rescue

/// S-box de Rescue, `x^7` (el ALPHA de `winter-crypto`, `rp64_256/mod.rs:52`).
fn sbox<E: FieldElement>(x: E) -> E {
    let x2 = x * x;
    let x4 = x2 * x2;
    x4 * x2 * x
}

/// **La restriccion de una ronda de Rescue**, por encuentro en el medio: `sbox(B) = A` con
/// `A = MDS * sbox(actual) + ark1` y `B = INV_MDS * (siguiente - ark2)`. Es la misma que llevan
/// en linea los circuitos de `stark-experiment` (deuda declarada: veintiocho copias); aqui vive
/// UNA vez y los tres carriles la llaman.
pub fn ronda_rescue<E: FieldElement<BaseField = BaseElement>>(
    actual: &[E],
    siguiente: &[E],
    ark1: &[E],
    ark2: &[E],
    bandera: E,
    salida: &mut [E],
) {
    let mut s = [E::ZERO; ESTADO];
    for j in 0..ESTADO {
        s[j] = sbox(actual[j]);
    }
    for i in 0..ESTADO {
        let mut a = E::ZERO;
        let mut b = E::ZERO;
        for j in 0..ESTADO {
            a += E::from(Rp64_256::MDS[i][j]) * s[j];
            b += E::from(Rp64_256::INV_MDS[i][j]) * (siguiente[j] - ark2[j]);
        }
        salida[i] = bandera * (sbox(b) - (a + ark1[i]));
    }
}

// ------------------------------------------------------------------ el AIR

pub struct EdadAir {
    context: AirContext<BaseElement>,
    pi: EdadPublicInputs,
    hojas: usize,
}

impl EdadAir {
    fn num_aserciones(pi: &EdadPublicInputs, hojas: usize) -> usize {
        4 + usize::from(pi.n >= 1) + usize::from((pi.n as usize) < hojas) + 8
    }
}

impl Air for EdadAir {
    type BaseField = BaseElement;
    type PublicInputs = EdadPublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        let hojas = 1usize << pub_inputs.m;
        let c = |d: usize| TransitionConstraintDegree::with_cycles(d, vec![CICLO]);
        let mut p = Vec::with_capacity(PRINCIPALES);
        // 1. las rondas de los tres carriles
        for _ in 0..3 * ESTADO {
            p.push(c(7));
        }
        // 2. capacidad cero de A y de B en la fila 0 del ciclo
        for _ in 0..8 {
            p.push(c(1));
        }
        // 3. el estado inicial de la hoja de meta en la fila 0
        for _ in 0..ESTADO {
            p.push(c(1));
        }
        // 4. (1 - vivo) * P = 0: toda hoja distinta de cero se cuenta como viva
        for _ in 0..4 {
            p.push(c(2));
        }
        // 5. fuera * P = 0 y fuera * vivo = 0: nada vivo por encima de la marca
        for _ in 0..5 {
            p.push(c(2));
        }
        // 6. la edad: qa = 0 exige T - 1 - (seq - nacido) en BITS bits
        p.push(c(2));
        // 7. el emisor: qs = 0 exige emisor distinto del nombrado
        p.push(if pub_inputs.todos { c(1) } else { c(3) });
        // 8. z = vivo * qa * qs
        p.push(c(3));
        // 9. booleanos: vivo, qa, qs, fuera y los bits
        for _ in 0..(4 + BITS) {
            p.push(TransitionConstraintDegree::new(2));
        }
        // 10. vivo y fuera constantes dentro del ciclo
        for _ in 0..2 {
            p.push(c(1));
        }
        // 11. fuera no baja en el enlace
        p.push(c(2));
        // 12. el contador de ciclo, act y la cuenta
        p.push(TransitionConstraintDegree::new(1));
        p.push(c(1));
        p.push(c(1));
        debug_assert_eq!(p.len(), PRINCIPALES);

        let mut x = Vec::with_capacity(AUXILIARES);
        // cinco lecturas/escritura en la fila 0 y dos escrituras en la 6: h * cod = 1
        for _ in 0..5 {
            x.push(c(2));
        }
        x.push(c(2));
        x.push(c(2));
        // la hoja de meta escrita es vivo * salida de M: un grado mas
        x.push(c(3));
        // la suma acumulada
        x.push(c(2));
        debug_assert_eq!(x.len(), AUXILIARES);

        let aserciones = Self::num_aserciones(&pub_inputs, hojas);
        EdadAir {
            context: AirContext::new_multi_segment(trace_info, p, x, aserciones, 2, options),
            pi: pub_inputs,
            hojas,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    /// Selector de hash (filas 0..6), selector de la fila 0, selector de la fila 6, y las
    /// constantes de ronda ARK1 y ARK2 de `Rp64_256`, todas de longitud CICLO.
    fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
        let mut cols = Vec::with_capacity(3 + 2 * ESTADO);
        let fila = |k: usize| {
            let mut v = vec![BaseElement::ZERO; CICLO];
            v[k] = BaseElement::ONE;
            v
        };
        let mut hash = vec![BaseElement::ONE; RONDAS];
        hash.push(BaseElement::ZERO);
        cols.push(hash);
        cols.push(fila(0));
        cols.push(fila(6));
        for i in 0..ESTADO {
            let mut v: Vec<BaseElement> = (0..RONDAS).map(|r| Rp64_256::ARK1[r][i]).collect();
            v.push(BaseElement::ZERO);
            cols.push(v);
        }
        for i in 0..ESTADO {
            let mut v: Vec<BaseElement> = (0..RONDAS).map(|r| Rp64_256::ARK2[r][i]).collect();
            v.push(BaseElement::ZERO);
            cols.push(v);
        }
        cols
    }

    fn evaluate_transition<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        periodic_values: &[E],
        result: &mut [E],
    ) {
        let c = frame.current();
        let s = frame.next();
        let hf = periodic_values[P_HASH];
        let f0 = periodic_values[P_F0];
        let enlace = E::ONE - hf;
        let ark1 = &periodic_values[P_ARK1..P_ARK1 + ESTADO];
        let ark2 = &periodic_values[P_ARK2..P_ARK2 + ESTADO];
        let mut i = 0;

        for base in [C_A, C_B, C_M] {
            let (a, b) = (&c[base..base + ESTADO], &s[base..base + ESTADO]);
            ronda_rescue(a, b, ark1, ark2, hf, &mut result[i..i + ESTADO]);
            i += ESTADO;
        }
        for k in 0..4 {
            result[i] = f0 * c[C_A + k];
            result[i + 4] = f0 * c[C_B + k];
            i += 1;
        }
        i += 4;
        let dominio = E::from(BaseElement::new(DOMINIO_META_PENDIENTE));
        for k in 0..ESTADO {
            let esperado = match k {
                0 => dominio,
                4 => c[C_EMISOR],
                5 => c[C_NACIDO],
                _ => E::ZERO,
            };
            result[i] = f0 * (c[C_M + k] - esperado);
            i += 1;
        }
        let vivo = c[C_VIVO];
        let fuera = c[C_FUERA];
        for k in 0..4 {
            result[i] = f0 * (E::ONE - vivo) * c[C_P + k];
            result[i + 4] = f0 * fuera * c[C_P + k];
            i += 1;
        }
        i += 4;
        result[i] = f0 * fuera * vivo;
        i += 1;

        let dos = E::from(2u32);
        let mut resto = E::ZERO;
        let mut peso = E::ONE;
        for b in 0..BITS {
            resto += c[C_BITS + b] * peso;
            peso *= dos;
        }
        let t = E::from(BaseElement::new(self.pi.t));
        let seq = E::from(BaseElement::new(self.pi.seq));
        result[i] = f0 * (E::ONE - c[C_QA]) * (t - E::ONE - seq + c[C_NACIDO] - resto);
        i += 1;

        let qs = c[C_QS];
        result[i] = if self.pi.todos {
            f0 * (E::ONE - qs)
        } else {
            let nombrado = E::from(BaseElement::new(self.pi.emisor));
            f0 * (E::ONE - qs) * ((c[C_EMISOR] - nombrado) * c[C_WINV] - E::ONE)
        };
        i += 1;
        result[i] = f0 * (c[C_Z] - vivo * c[C_QA] * qs);
        i += 1;

        for col in [C_VIVO, C_QA, C_QS, C_FUERA] {
            result[i] = c[col] * (c[col] - E::ONE);
            i += 1;
        }
        for b in 0..BITS {
            let x = c[C_BITS + b];
            result[i] = x * (x - E::ONE);
            i += 1;
        }
        result[i] = hf * (s[C_VIVO] - vivo);
        result[i + 1] = hf * (s[C_FUERA] - fuera);
        result[i + 2] = enlace * fuera * (E::ONE - s[C_FUERA]);
        result[i + 3] = s[C_CICLO] - c[C_CICLO] - enlace;
        result[i + 4] = s[C_ACT] - c[C_ACT] - enlace * (E::ONE - c[C_ACT]);
        result[i + 5] = s[C_CUENTA] - c[C_CUENTA] - f0 * c[C_Z];
        debug_assert_eq!(i + 6, PRINCIPALES);
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let ultima = self.trace_length() - 1;
        let n = self.pi.n as usize;
        let mut a = vec![
            Assertion::single(C_CICLO, 0, BaseElement::ZERO),
            Assertion::single(C_ACT, 0, BaseElement::ZERO),
            Assertion::single(C_CUENTA, 0, BaseElement::ZERO),
            Assertion::single(C_CUENTA, ultima, BaseElement::new(self.pi.k)),
        ];
        if n >= 1 {
            a.push(Assertion::single(C_FUERA, CICLO * (n - 1), BaseElement::ZERO));
        }
        if n < self.hojas {
            a.push(Assertion::single(C_FUERA, CICLO * n, BaseElement::ONE));
        }
        // el nodo 1 sale del ciclo 1, en su ultima fila
        let raiz = 2 * CICLO - 1;
        for k in 0..4 {
            a.push(Assertion::single(C_A + 4 + k, raiz, self.pi.subraiz_pend[k]));
            a.push(Assertion::single(C_B + 4 + k, raiz, self.pi.subraiz_meta[k]));
        }
        a
    }

    fn evaluate_aux_transition<F, E>(
        &self,
        main_frame: &EvaluationFrame<F>,
        aux_frame: &EvaluationFrame<E>,
        periodic_values: &[F],
        aux_rand_elements: &AuxRandElements<E>,
        result: &mut [E],
    ) where
        F: FieldElement<BaseField = Self::BaseField>,
        E: FieldElement<BaseField = Self::BaseField> + ExtensionOf<F>,
    {
        let c = main_frame.current();
        let s = main_frame.next();
        let x = aux_frame.current();
        let y = aux_frame.next();
        let f0 = E::from(periodic_values[P_F0]);
        let f6 = E::from(periodic_values[P_F6]);
        let r = aux_rand_elements.rand_elements();
        let pot = potencias(r[0]);
        let beta = r[1];
        let dig = |fila: &[F], col: usize| -> [E; 4] {
            [
                E::from(fila[col]),
                E::from(fila[col + 1]),
                E::from(fila[col + 2]),
                E::from(fila[col + 3]),
            ]
        };
        let hojas = E::from(BaseElement::new(self.hojas as u64));
        let ciclo = E::from(c[C_CICLO]);
        let izq = E::from(2u32) * ciclo;
        let der = izq + E::ONE;
        let (ca, cb) = (carril::<E>(false), carril::<E>(true));
        let act = E::from(c[C_ACT]);
        let vivo = E::from(c[C_VIVO]);

        let lecturas = [
            codificar(&pot, beta, izq, dig(c, C_A + 4), ca),
            codificar(&pot, beta, der, dig(c, C_A + 8), ca),
            codificar(&pot, beta, izq, dig(c, C_B + 4), cb),
            codificar(&pot, beta, der, dig(c, C_B + 8), cb),
            codificar(&pot, beta, hojas + ciclo, dig(c, C_P), ca),
        ];
        let m = dig(s, C_M + 4);
        let hoja_meta = [vivo * m[0], vivo * m[1], vivo * m[2], vivo * m[3]];
        let escrituras = [
            codificar(&pot, beta, ciclo, dig(s, C_A + 4), ca),
            codificar(&pot, beta, ciclo, dig(s, C_B + 4), cb),
            codificar(&pot, beta, hojas + ciclo, hoja_meta, cb),
        ];
        for k in 0..5 {
            result[k] = f0 * (x[X_H + k] * lecturas[k] - E::ONE);
        }
        for k in 0..3 {
            result[5 + k] = f6 * (x[X_H + k] * escrituras[k] - E::ONE);
        }
        let leidas = x[X_H] + x[X_H + 1] + x[X_H + 2] + x[X_H + 3];
        let escritas = act * (x[X_H] + x[X_H + 1]) + x[X_H + 2];
        result[8] = y[X_S] - x[X_S] - f0 * (x[X_H + 4] - act * leidas) - f6 * escritas;
    }

    fn get_aux_assertions<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        aux_rand_elements: &AuxRandElements<E>,
    ) -> Vec<Assertion<E>> {
        let r = aux_rand_elements.rand_elements();
        let pot = potencias(r[0]);
        let e = |d: Digest| [E::from(d[0]), E::from(d[1]), E::from(d[2]), E::from(d[3])];
        let a = codificar(&pot, r[1], E::ONE, e(self.pi.subraiz_pend), carril::<E>(false));
        let b = codificar(&pot, r[1], E::ONE, e(self.pi.subraiz_meta), carril::<E>(true));
        vec![
            Assertion::single(X_S, 0, E::ZERO),
            Assertion::single(X_S, self.trace_length() - 1, a.inv() + b.inv()),
        ]
    }
}

// ------------------------------------------------------------------ el juez

/// **Verifica una prueba de edad contra su enunciado.** Comprueba ANTES la forma de la traza que
/// la prueba declara -anchos, aleatorios y longitud `8 * 2^m`- para que un fichero ajeno no llegue
/// a construir el AIR, y acepta SOLO [`opciones`]. Nunca entra en panico por lo que traiga la
/// prueba.
pub fn verificar(prueba: &[u8], pi: &EdadPublicInputs) -> Result<(), String> {
    comprobar_enunciado(pi)?;
    let proof =
        Proof::from_bytes(prueba).map_err(|e| format!("la prueba no se deserializa: {e:?}"))?;
    let info = proof.trace_info();
    let filas = CICLO << pi.m;
    let forma = (
        info.main_trace_width(),
        info.aux_segment_width(),
        info.get_num_aux_segment_rand_elements(),
        info.length(),
    );
    if forma != (ANCHO, ANCHO_AUX, ALEATORIOS, filas) {
        return Err(format!(
            "forma de traza {forma:?}; el enunciado pide {:?}",
            (ANCHO, ANCHO_AUX, ALEATORIOS, filas)
        ));
    }
    let aceptadas = AcceptableOptions::OptionSet(vec![opciones()]);
    verify::<EdadAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
        proof,
        pi.clone(),
        &aceptadas,
    )
    .map_err(|e| format!("{e:?}"))
}

// ------------------------------------------------------------------ las raices nativas

/// Los subarboles vacios: `vacio[0]` es la hoja cero y `vacio[k] = merge(vacio[k-1], vacio[k-1])`,
/// la misma convencion que `SparseTree` de la capa.
pub fn vacios(profundidad: usize) -> Vec<Digest> {
    let mut v = vec![[BaseElement::ZERO; 4]];
    for k in 1..=profundidad {
        v.push(native_merge(v[k - 1], v[k - 1]));
    }
    v
}

/// La raiz de un subarbol completo de `hojas.len()` hojas (una potencia de dos, al menos 2).
pub fn subraiz(hojas: &[Digest]) -> Digest {
    assert!(hojas.len() >= 2 && hojas.len().is_power_of_two(), "hojas: potencia de dos >= 2");
    let mut nivel = hojas.to_vec();
    while nivel.len() > 1 {
        nivel = nivel.chunks(2).map(|p| native_merge(p[0], p[1])).collect();
    }
    nivel[0]
}

/// **La subida del juez**: de la subraiz de `[0, 2^m)` a la raiz de `profundidad` niveles, con el
/// subarbol vacio de cada nivel como hermano derecho. Una posicion viva en `[2^m, 2^profundidad)`
/// hace que esta raiz no sea la firmada.
pub fn raiz_desde_subraiz(sub: Digest, m: u32, profundidad: usize) -> Digest {
    let v = vacios(profundidad);
    let mut acc = sub;
    for k in (m as usize)..profundidad {
        acc = native_merge(acc, v[k]);
    }
    acc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(k: u64) -> Digest {
        [
            BaseElement::new(k),
            BaseElement::new(3 * k + 1),
            BaseElement::new(7 * k + 2),
            BaseElement::new(11 * k + 3),
        ]
    }

    fn enunciado() -> EdadPublicInputs {
        EdadPublicInputs {
            subraiz_pend: d(1),
            subraiz_meta: d(2),
            m: 4,
            n: 13,
            seq: 100,
            t: 10,
            emisor: 7,
            todos: false,
            k: 3,
        }
    }

    /// Cada ranura declarada se escribe: con un centinela en `result`, ninguna sobrevive a una
    /// evaluacion sobre un marco cualquiera, y el numero de ranuras es el que el contexto cuenta.
    #[test]
    fn las_restricciones_declaradas_son_las_escritas() {
        let pi = enunciado();
        let filas = CICLO << pi.m;
        let info = TraceInfo::new_multi_segment(ANCHO, ANCHO_AUX, ALEATORIOS, filas, vec![]);
        let air = EdadAir::new(info, pi, opciones());
        assert_eq!(air.context().num_main_transition_constraints(), PRINCIPALES);
        assert_eq!(air.context().num_aux_transition_constraints(), AUXILIARES);
        let centinela = BaseElement::new(0xDEAD_BEEF);
        let fila = |k: u64| -> Vec<BaseElement> {
            (0..ANCHO as u64).map(|i| BaseElement::new(k * 1000 + i + 5)).collect()
        };
        let marco = EvaluationFrame::from_rows(fila(1), fila(2));
        let periodicas: Vec<BaseElement> =
            (0..3 + 2 * ESTADO as u64).map(|i| BaseElement::new(i + 2)).collect();
        let mut r = vec![centinela; PRINCIPALES];
        air.evaluate_transition(&marco, &periodicas, &mut r);
        assert!(r.iter().all(|x| *x != centinela), "una ranura principal sin escribir");
        let aux_fila = |k: u64| -> Vec<BaseElement> {
            (0..ANCHO_AUX as u64).map(|i| BaseElement::new(k * 77 + i + 9)).collect()
        };
        let aux = EvaluationFrame::from_rows(aux_fila(3), aux_fila(4));
        let al = AuxRandElements::new(vec![BaseElement::new(31), BaseElement::new(37)]);
        let mut x = vec![centinela; AUXILIARES];
        air.evaluate_aux_transition(&marco, &aux, &periodicas, &al, &mut x);
        assert!(x.iter().all(|v| *v != centinela), "una ranura auxiliar sin escribir");
    }

    /// La subida del juez da la raiz del arbol entero: un arbol de 16 hojas con las 4 primeras
    /// ocupadas, recompuesto hoja a hoja, contra la subraiz de las 4 subida a 4 niveles.
    #[test]
    fn la_raiz_desde_la_subraiz_es_la_del_arbol_entero() {
        let cuatro = [d(10), d(11), [BaseElement::ZERO; 4], d(13)];
        let mut dieciseis = vec![[BaseElement::ZERO; 4]; 16];
        dieciseis[..4].copy_from_slice(&cuatro);
        assert_eq!(raiz_desde_subraiz(subraiz(&cuatro), 2, 4), subraiz(&dieciseis));
        dieciseis[9] = d(99);
        assert_ne!(
            raiz_desde_subraiz(subraiz(&cuatro), 2, 4),
            subraiz(&dieciseis),
            "una viva fuera del subarbol paso desapercibida"
        );
    }

    /// Un enunciado fuera de rango y una prueba que no es prueba se rechazan con `Err`, sin
    /// construir el AIR ni entrar en panico.
    #[test]
    fn lo_que_no_tiene_forma_se_rechaza_sin_panico() {
        let bien = enunciado();
        for malo in [
            EdadPublicInputs { m: 0, ..bien.clone() },
            EdadPublicInputs { m: 25, ..bien.clone() },
            EdadPublicInputs { n: 17, ..bien.clone() },
            EdadPublicInputs { t: 1 << 32, ..bien.clone() },
            EdadPublicInputs { seq: 1 << 32, ..bien.clone() },
            EdadPublicInputs { k: 14, ..bien.clone() },
        ] {
            assert!(comprobar_enunciado(&malo).is_err(), "aceptado: {malo:?}");
        }
        assert!(comprobar_enunciado(&bien).is_ok());
        assert!(verificar(&[1, 2, 3], &bien).is_err(), "tres bytes verificaron");
    }

    /// Las quince entradas publicas van al transcripto, cada una en su sitio.
    #[test]
    fn las_entradas_publicas_van_enteras_al_transcripto() {
        let pi = enunciado();
        let v = pi.to_elements();
        assert_eq!(v.len(), 15);
        assert_eq!(&v[0..4], &pi.subraiz_pend);
        assert_eq!(&v[4..8], &pi.subraiz_meta);
        let resto: Vec<u64> = v[8..].iter().map(|x| x.as_int()).collect();
        assert_eq!(resto, vec![4, 13, 100, 10, 7, 0, 3]);
    }
}
