# halo2-experiment

Crate aislado, separado del resto del workspace (`zk-core`, `iso-bridge`),
para probar si Halo2 es una vía viable como reemplazo de Groth16 — sin
arriesgar nada del código ya verificado.

## Estado

**Sin compilar todavía.** Es la pieza de mayor incertidumbre de todo el
proyecto — más que `poseidon_hash.rs` en su momento. Espera varias rondas
de corrección.

## Qué hace

`SquareCircuit`: demuestra conocimiento de un valor privado `x` tal que
`x * x = y`, con `y` público — el ejemplo mínimo canónico de Halo2, sin
tocar Poseidon, árboles de Merkle ni nullifiers todavía.

Los tests usan `MockProver` (comprobación de satisfacibilidad, no
generación de pruebas reales) — deliberadamente, para aislar el primer
posible punto de fallo (la definición del circuito) del segundo (el
pipeline completo de prueba/verificación con parámetros IPA), que es
aún más incierto y se abordará en una ronda posterior si esto compila.

## Cómo compilar y testear

```bash
cargo test -p halo2-experiment -- --nocapture
```

Si falla, pega el error completo — la primera sospecha son los nombres
exactos de métodos de la API de `ConstraintSystem`/`Layouter`/`Region`,
que no he podido verificar sin compilar.
