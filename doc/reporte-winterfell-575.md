# Borrador de divulgación responsable a winterfell — la deuda del §575

> **Para el autor, no para el árbol público como aviso a terceros.** Este es el borrador del
> reporte que `SECURITY.md` 3.7 y el §575 dejaron como deuda: *«winterfell upstream tiene el mismo
> defecto, y avisarle es deuda»*. Lo redactó un asistente con el método de `GENAI.md`; enviarlo y
> aceptarlo es del autor.

## Por qué canal, y cuál al final — MEDIDO el 01-10-2026

El fallo es de denegación de servicio en la deserialización de datos ajenos, así que no va como
issue ni PR público mientras siga sin arreglar aguas arriba. Buscando el canal privado se midió
esto, y no se dio por supuesto:

- **El aviso privado de GitHub NO está habilitado** en `facebook/winterfell`: no hay botón «Report a
  vulnerability». Esa puerta, la que no pide Facebook, está cerrada, y sólo el dueño del repo la abre.
- **No hay `SECURITY.md` propio** en el árbol de winterfell (`HEAD` en `2f78ee9`, la 0.13.1). La
  política que GitHub muestra —reportar por el programa de Meta en `facebook.com/whitehat`— es la
  **plantilla por defecto de la organización Facebook**, heredada, no una del proyecto. Y ese
  programa exige una cuenta de Facebook.
- **El mantenedor, con correo verificable:** quien publicó la 0.13.1 (commit `2f78ee9`, 19-07-2025)
  es **Irakliy Khaburzaniya**, que commitea con un correo público, `irakliy81@gmail.com`.

Sin Facebook, la vía honesta que queda para un fallo sin arreglar es el **correo privado directo al
mantenedor**. Es lo que se hizo: reporte enviado el **01-10-2026** a `irakliy81@gmail.com`, con la
causa y la vía, el arreglo ofrecido como PR, y **sin reproductor**. El repositorio lleva sin un
commit en `main` desde el 19-07-2025 y su equipo se movió a Plonky3 (ver
`doc/integracion-vertical-evaluacion.md`, §4), así que puede no haber respuesta: el correo fija
**90 días**, y pasados sin respuesta la escalada de divulgación coordinada es un aviso en RustSec
(`rustsec.org`), nunca un issue público en winterfell mientras siga sin arreglar. `AUDITORIA.md` §625.

## El fallo, en una frase

Al deserializar una prueba o un lote de Merkle, winterfell reserva memoria para tantos elementos
como diga una longitud leída de los propios bytes, **antes** de comprobar que esos bytes existen.
Una entrada malformada que declare una longitud enorme provoca una reserva que falla, y una
reserva que falla **aborta el proceso** en vez de devolver un error: no es un pánico, así que un
consumidor no la recoge con `catch_unwind`. Cualquiera que deserialice una prueba de origen no
fiable puede tumbar el proceso que la verifica.

## Qué está confirmado, y sobre qué

Medido el 01-10-2026, con rustc 1.97.0, contra los crates **publicados en crates.io** (no un fork):
`winter-air`, `winter-utils`, `winter-crypto` y `winter-math`, los cuatro clavados a `=0.13.1`.
Tres vías de entrada quedan confirmadas, cada una abortando el proceso con un mensaje de reserva de
memoria fallida, mientras la misma llamada con una longitud honesta devuelve `Err(UnexpectedEOF)`:

- La deserialización genérica de un `Vec` (`winter-utils`, `ByteReader::read_many`, por donde pasa
  toda `Vec` que se lee).
- `Proof::from_bytes` (`winter-air`), por la primera longitud que lee una `Queries`.
- `BatchMerkleProof::read_from` (`winter-crypto`), por su cuenta de vectores de nodos.

La causa común está en `winter-utils 0.13.1`: `read_many` hace `Vec::with_capacity(num_elements)`
antes de leer ni un elemento, y `num_elements` sale de los bytes. El `check_eor` del `SliceReader`
no ayuda porque nadie lo llama antes de esa reserva.

## El arreglo, que ya existe y está medido

Arqueo lo cerró en su copia en el asiento §575 (público en `SECURITY.md` 3.7 y en el README de
`crates/winter-air`): un lector que comprueba que caben los elementos —al menos un byte cada uno—
**antes** de reservar, de modo que más elementos que bytes restantes es `UnexpectedEOF`, el mismo
error que upstream ya da al quedarse sin bytes. Una prueba bien formada se lee igual, byte a byte.
Es el patrón que varias bibliotecas de deserialización aplican: acotar la cuenta por los bytes
disponibles antes de reservar.

El texto del reporte puede describir ese arreglo; el código concreto, si lo piden, se ofrece como
pull request por el canal que indiquen.

## Lo que el reporte NO debe incluir

- Un guion listo para tumbar un despliegue: basta describir la causa y la vía, que es lo que un
  mantenedor necesita para arreglarlo y reproducirlo él.
- Nada sobre despliegues concretos de terceros: el reporte es sobre la biblioteca, no sobre quién
  la usa.
