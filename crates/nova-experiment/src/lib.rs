//! Quinto backend: **Nova / esquemas de plegado (folding)** sobre el
//! ciclo de curvas BN254/Grumpkin.
//!
//! ## Por qué esto NO es "el mismo circuito por quinta vez"
//!
//! Los otros cuatro backends (Groth16, Halo2/IPA, STARK/FRI, PLONK/KZG)
//! producen una **prueba monolítica** del circuito entero. Nova hace algo
//! distinto: **pliega** una secuencia de pasos, acumulando el estado sin
//! recalcular nada, y comprime al final con un SNARK.
//!
//! Eso significa que probar UNA sola transacción con Nova sería más lento
//! que con cualquiera de los otros — se paga la maquinaria de plegado sin
//! amortizarla. La pregunta que Nova responde y los demás no es otra:
//!
//! > **¿cuánto cuesta la transacción número N+1?**
//!
//! En una cámara de compensación que liquida miles de operaciones al día
//! y cierra al final de la jornada, esa es la pregunta relevante.
//!
//! ## El encaje con la partida doble es natural, no forzado
//!
//! El trait de Nova es:
//!
//! ```text
//! fn synthesize(&self, cs, z_i) -> z_{i+1}
//! ```
//!
//! Es decir: un circuito que transforma un estado en el siguiente. Y una
//! transferencia de partida doble es exactamente eso — entra `root_i`,
//! sale `root_{i+1}`. El trabajo de los otros cuatro backends encaja aquí
//! sin violentarlo.
//!
//! ## Lo que Nova NO da por sí solo
//!
//! Nova produce un **estado plegado**, no una prueba entregable a un
//! tercero. Para cerrarla hay que comprimirla con un SNARK (Spartan, que
//! el propio crate incluye). Por eso Nova no sustituye a los otros
//! backends: los complementa.
//!
//! ## El hallazgo más notable de este backend
//!
//! **`nova-snark` se NIEGA a hacer un setup de una sola parte en
//! compilaciones de producción.** El mensaje es literal:
//!
//! ```text
//! HyperKZG::setup is disabled in production builds. Use
//! PublicParams::setup_with_ptau_dir ... with ptau files from a trusted
//! setup ceremony. For tests, enable the 'test-utils' feature.
//! ```
//!
//! Es la **única de las cinco librerías de este proyecto que impide en
//! código** lo que las otras cuatro se limitan a desaconsejar en la
//! documentación. Groth16, Halo2, STARK y PLONK-KZG permiten generar
//! parámetros con un `setup()` local y confían en que quien lo use lo
//! documente — este proyecto lo ha hecho a mano cinco veces.
//!
//! Y ofrece la vía de producción resuelta: `setup_with_ptau_dir` consume
//! ficheros ptau de **Perpetual Powers of Tau**, la ceremonia pública
//! existente. Es exactamente la propiedad que en PLONK-KZG costó seis
//! vías de investigación confirmar, y aquí viene documentada en el propio
//! mensaje de error.
//!
//! Los tests de este crate habilitan `test-utils` deliberadamente, y esa
//! feature NO está activada en la compilación normal.
//!
//! ## Estado
//!
//! - `folding_chain`: cadena de plegado mínima, con medición del coste
//!   marginal por paso. ✅ verificado 3/3.
//!
//! ## Los números medidos, y lo que demuestran
//!
//! | Fase | Coste |
//! |---|---|
//! | `PublicParams::setup` (una vez) | 4,02 s |
//! | **Por transacción (`prove_step`)** | **~250 ms, CONSTANTE** |
//! | `RecursiveSNARK::verify` | 108 ms |
//! | `CompressedSNARK::prove` (cierre) | 1,84 s |
//! | `CompressedSNARK::verify` | 50 ms |
//!
//! El coste del paso 9 fue **0,77 veces** el del paso 1 — es decir, no
//! crece con la longitud de la cadena. Esa es la propiedad que define a
//! Nova y que ninguno de los otros cuatro backends tiene.
//!
//! El perfil resultante —barato durante el día, caro al cerrar— es
//! exactamente el de una cámara de compensación que liquida operaciones
//! durante la jornada y cierra por la noche.
//!
//! ## ⚠️ Matiz honesto sobre estas cifras
//!
//! El paso de esta pieza hace **un solo hash Poseidon**, y aun así
//! consume **10.764 restricciones**. Casi todo es sobrecoste del propio
//! plegado: el "circuito verificador" que Nova inserta en cada paso.
//!
//! Por tanto **los ~250 ms NO son comparables** con el 1,17 s de la
//! partida doble en Groth16. Un paso con partida doble real rondaría las
//! 38.000 restricciones (27.562 del circuito más el sobrecoste) y
//! costaría bastante más.
//!
//! Lo demostrado aquí es que **el mecanismo de plegado funciona y su
//! coste es constante**, no que Nova sea más rápido para este caso de
//! uso. Comparar de verdad exigiría portar la partida doble al
//! `StepCircuit`.

pub mod folding_chain;
