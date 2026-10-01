//! The draft's accumulated vectors (appendix "Test Vectors"): for every
//! subtree of every tree up to 130 leaves, a running hash over the outputs
//! of each algorithm: 712 subtree hashes, 12,807 inclusion proofs, 42,893
//! consistency proofs and 8,646 coverings, 65,058 cases in total. If a
//! single one differs, the final hash does not match.
//!
//! ADAPTADO (§631): copiado de `tests/vectors.rs` de mtc-core `d3b0ca6`.
//! Cambian el nombre del crate en los `use`, `hex` (allí `der::hex`, que no
//! se copia) y el último test: allí `a_proof_off_by_one_byte_does_not_decode`
//! recortaba y alargaba un byte de un `MtcProof` codificado, que es del
//! formato MTC y no del medio; aquí queda su segunda mitad, la prueba de
//! otro índice, que sí es del árbol.

use sha2::{Digest, Sha256};
use zk_ssl_medio::hash::hash_leaf;
use zk_ssl_medio::subtree::{
    consistency_proof, covering_subtrees, evaluate_inclusion_proof, inclusion_proof,
    is_valid_subtree, verify_consistency_proof, verify_inclusion_proof, LeafHashes, Subtree,
    SubtreeError, TreeHashes,
};

/// ADAPTADO (§631): en mtc-core, `der::hex`.
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

const N: u64 = 130;

/// `d[i] = bytes([i])`, already as hashed leaves.
fn tree() -> LeafHashes {
    LeafHashes((0..N).map(|i| hash_leaf(&[i as u8])).collect())
}

#[test]
fn subtree_hashes() {
    let t = tree();
    let mut h = Sha256::new();
    for end in 0..=N {
        for start in 0..=end {
            if is_valid_subtree(start, end) {
                h.update(format!(
                    "[{start}, {end}) {}\n",
                    hex(&t.range_hash(start, end))
                ));
            }
        }
    }
    assert_eq!(
        hex(&h.finalize()),
        "b82806ad4265bb151c1119c0f4db437bb4d1a1f887b3a7fba1cd4ebf552e3e81"
    );
}

#[test]
fn subtree_inclusion_proofs() {
    let t = tree();
    let mut h = Sha256::new();
    for end in 0..=N {
        for start in 0..=end {
            if !is_valid_subtree(start, end) {
                continue;
            }
            let st = Subtree { start, end };
            let subtree_hash = t.range_hash(start, end);
            for index in start..end {
                let proof = inclusion_proof(&t, st, index).unwrap();
                let mut line = format!("{index} [{start}, {end})");
                for p in &proof {
                    line.push(' ');
                    line.push_str(&hex(p));
                }
                line.push('\n');
                h.update(line);

                // The verifier exercise: evaluate, and reject truncations and additions.
                assert_eq!(
                    evaluate_inclusion_proof(&t.0[index as usize], st, index, &proof).unwrap(),
                    subtree_hash
                );
                if !proof.is_empty() {
                    assert!(evaluate_inclusion_proof(
                        &t.0[index as usize],
                        st,
                        index,
                        &proof[..proof.len() - 1]
                    )
                    .is_err());
                }
                let mut longer = proof.clone();
                longer.push([0x5a; 32]);
                assert!(
                    evaluate_inclusion_proof(&t.0[index as usize], st, index, &longer).is_err()
                );
            }
        }
    }
    assert_eq!(
        hex(&h.finalize()),
        "ac2a8f989e44d99e399db448050ff5f19757df53cfb716aa81015d3955d8163f"
    );
}

#[test]
fn subtree_consistency_proofs() {
    let t = tree();
    let mut h = Sha256::new();
    for n in 0..=N {
        let root = t.range_hash(0, n);
        for end in 0..=n {
            for start in 0..=end {
                if !is_valid_subtree(start, end) {
                    continue;
                }
                let st = Subtree { start, end };
                let proof = consistency_proof(&t, n, st).unwrap();
                let mut line = format!("[{start}, {end}) {n}");
                for p in &proof {
                    line.push(' ');
                    line.push_str(&hex(p));
                }
                line.push('\n');
                h.update(line);

                // The verifier exercise.
                let node = t.range_hash(start, end);
                verify_consistency_proof(n, st, &proof, &node, &root).unwrap();
                if !proof.is_empty() {
                    assert!(verify_consistency_proof(
                        n,
                        st,
                        &proof[..proof.len() - 1],
                        &node,
                        &root
                    )
                    .is_err());
                }
                let mut longer = proof.clone();
                longer.push([0x5a; 32]);
                assert!(verify_consistency_proof(n, st, &longer, &node, &root).is_err());
                let mut flipped = node;
                flipped[0] ^= 1;
                assert!(verify_consistency_proof(n, st, &proof, &flipped, &root).is_err());
                if start != end {
                    let mut flipped = root;
                    flipped[31] ^= 1;
                    assert!(verify_consistency_proof(n, st, &proof, &node, &flipped).is_err());
                }
            }
        }
    }
    assert_eq!(
        hex(&h.finalize()),
        "10fa99b37bf9bf9ffa26b412fbd98bd75363256d0b75d61bc4538b9c9c5a0a74"
    );
}

#[test]
fn efficient_covering_subtrees() {
    let mut h = Sha256::new();
    for end in 0..=N {
        for start in 0..=end {
            let (l, r) = covering_subtrees(start, end);
            assert!(is_valid_subtree(l.start, l.end) && is_valid_subtree(r.start, r.end));
            assert!(
                l.start <= start
                    && start <= l.end
                    && l.end == r.start
                    && r.start <= end
                    && end == r.end
            );
            h.update(format!(
                "[{}, {}) [{}, {})\n",
                l.start, l.end, r.start, r.end
            ));
        }
    }
    assert_eq!(
        hex(&h.finalize()),
        "7fd9c8b926e9d2b5cf831560e8ce295a5ef97ad5c5ede4ea0dea28a8c8fc8bb0"
    );
}

#[test]
fn large_subtree_validity() {
    for (start, end) in [
        (0, (1u64 << 47) + 1),
        (0, (1u64 << 48) - 1),
        (0, (1u64 << 62) + 1),
        (0, (1u64 << 63) - 1),
        (0, (1u64 << 63) + 1),
        (0, u64::MAX),
    ] {
        assert!(is_valid_subtree(start, end), "[{start}, {end})");
    }
    for (start, end) in [
        (1u64 << 46, (1u64 << 47) + 1),
        (1u64 << 46, (1u64 << 48) - 1),
        (1u64 << 61, (1u64 << 62) + 1),
        (1u64 << 61, (1u64 << 63) - 1),
        (1u64 << 62, (1u64 << 63) + 1),
        (1u64 << 62, u64::MAX),
    ] {
        assert!(!is_valid_subtree(start, end), "[{start}, {end})");
    }
}

/// `(start, end, left, right)`.
type CoveringCase = (u64, u64, (u64, u64), (u64, u64));

#[test]
fn large_covering_subtrees() {
    let cases: [CoveringCase; 15] = [
        (
            0x0,
            0x800000000000,
            (0x0, 0x400000000000),
            (0x400000000000, 0x800000000000),
        ),
        (
            0x500000000000,
            0xd00000000000,
            (0x400000000000, 0x800000000000),
            (0x800000000000, 0xd00000000000),
        ),
        (
            0x7fffffffffff,
            0x800000000001,
            (0x7fffffffffff, 0x800000000000),
            (0x800000000000, 0x800000000001),
        ),
        (
            0xfffffffffffe,
            0xffffffffffff,
            (0xfffffffffffe, 0xffffffffffff),
            (0xffffffffffff, 0xffffffffffff),
        ),
        (
            0xffffffffffff,
            0xffffffffffff,
            (0xffffffffffff, 0xffffffffffff),
            (0xffffffffffff, 0xffffffffffff),
        ),
        (
            0x0,
            0x4000000000000000,
            (0x0, 0x2000000000000000),
            (0x2000000000000000, 0x4000000000000000),
        ),
        (
            0x2800000000000000,
            0x6800000000000000,
            (0x2000000000000000, 0x4000000000000000),
            (0x4000000000000000, 0x6800000000000000),
        ),
        (
            0x3fffffffffffffff,
            0x4000000000000001,
            (0x3fffffffffffffff, 0x4000000000000000),
            (0x4000000000000000, 0x4000000000000001),
        ),
        (
            0x7ffffffffffffffe,
            0x7fffffffffffffff,
            (0x7ffffffffffffffe, 0x7fffffffffffffff),
            (0x7fffffffffffffff, 0x7fffffffffffffff),
        ),
        (
            0x7fffffffffffffff,
            0x7fffffffffffffff,
            (0x7fffffffffffffff, 0x7fffffffffffffff),
            (0x7fffffffffffffff, 0x7fffffffffffffff),
        ),
        (
            0x0,
            0x8000000000000000,
            (0x0, 0x4000000000000000),
            (0x4000000000000000, 0x8000000000000000),
        ),
        (
            0x5000000000000000,
            0xd000000000000000,
            (0x4000000000000000, 0x8000000000000000),
            (0x8000000000000000, 0xd000000000000000),
        ),
        (
            0x7fffffffffffffff,
            0x8000000000000001,
            (0x7fffffffffffffff, 0x8000000000000000),
            (0x8000000000000000, 0x8000000000000001),
        ),
        (
            0xfffffffffffffffe,
            0xffffffffffffffff,
            (0xfffffffffffffffe, 0xffffffffffffffff),
            (0xffffffffffffffff, 0xffffffffffffffff),
        ),
        (
            0xffffffffffffffff,
            0xffffffffffffffff,
            (0xffffffffffffffff, 0xffffffffffffffff),
            (0xffffffffffffffff, 0xffffffffffffffff),
        ),
    ];
    for (start, end, l, r) in cases {
        assert_eq!(
            covering_subtrees(start, end),
            (
                Subtree {
                    start: l.0,
                    end: l.1
                },
                Subtree {
                    start: r.0,
                    end: r.1
                }
            ),
            "[{start:#x}, {end:#x})"
        );
    }
}

/// ADAPTADO (§631): la segunda mitad de `a_proof_off_by_one_byte_does_not_decode`
/// de mtc-core; la primera, sobre la codificación de `MtcProof`, no se copia.
#[test]
fn a_proof_of_another_index_does_not_verify() {
    let t = tree();
    let st = Subtree { start: 8, end: 13 };
    // A proof for another index climbs to a different hash.
    let other = inclusion_proof(&t, st, 11).unwrap();
    assert_eq!(
        verify_inclusion_proof(&t.0[10], st, 10, &other, &t.range_hash(8, 13)),
        Err(SubtreeError::HashMismatch)
    );
}
