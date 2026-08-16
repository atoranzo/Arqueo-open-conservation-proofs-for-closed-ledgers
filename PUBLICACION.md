# Textos para acompañar la publicación

Cuatro versiones del mismo mensaje, según dónde se publique. **Adáptalos**:
están escritos para que suenen a ti, no a un anuncio.

Una regla que atraviesa las cuatro: **lo diferenciado de este trabajo no
es la capa, son las mediciones y los hallazgos**. Vender la capa como
infraestructura invita a que la comparen con Zcash o Aztec y salga
perdiendo. Presentarla como el artefacto sobre el que se hizo la
comparativa es exacto y no compite con nadie.

---

## 1. Foro técnico o comunidad de criptografía

> Implementé el mismo circuito de liquidación en cinco sistemas de prueba
> —Groth16, Halo2/IPA, STARK/FRI, PLONK/KZG y Nova— para ver qué cambia
> de verdad al portar una aplicación completa entre paradigmas, y no solo
> un SHA-256 de referencia.
>
> Algunos resultados que no esperaba:
>
> - **AIR no tiene restricciones de copia.** Al portar la actualización de
>   estado a STARK apareció un agujero que no existe en Plonkish: nada
>   obliga a que las dos subidas del árbol usen los mismos hermanos. Es
>   silencioso —un testigo honesto nunca lo revela— y obliga a un patrón
>   en lockstep.
> - **PLONK-KZG resultó el generador más lento** de los cuatro basados en
>   curvas, 16-22× más que Groth16. Parte puede ser de la implementación y
>   los datos no permiten separarlo.
> - **Goldilocks es demasiado estrecho para identidades**: 64 bits son
>   colisión en 2³².
> - Solo **dos de seis librerías impiden en código el uso inseguro**.
>
> Todo medido en la misma máquina y en release, con el código para
> reproducirlo. El artefacto sobre el que se midió es una capa de
> liquidación completa —privacidad, revelación selectiva, emisión con
> umbral, congelación— con sus límites declarados en cabecera: es un nodo
> único, y el operador ve todos los saldos.
>
> [enlace]

**Por qué funciona aquí**: abre con el método, da hallazgos concretos que
alguien puede discutir, y declara la limitación sin que se la tengan que
encontrar.

---

## 2. Red profesional

> He publicado el trabajo de varios meses: una capa de liquidación con
> pruebas de conocimiento cero donde las transferencias son privadas pero
> el cumplimiento normativo es demostrable, construida sin ninguna
> ceremonia de confianza.
>
> Lo que más me interesa compartir no es la capa sino **las mediciones**.
> Implementé el mismo circuito en cinco paradigmas de prueba distintos
> para poder comparar con datos en vez de con literatura. Verificar una
> liquidación cuesta el 0,5% de generarla; el arranque no necesita generar
> ninguna clave; y mil transferencias acumulan 126,2 MiB de pruebas.
>
> Y hay un límite que muerde antes: la posición de un nullifier se deriva
> del propio nullifier, así que por la paradoja del cumpleaños, a los
> ~65.000 pagos las colisiones ya son probables, y el afectado no puede
> reintentar.
>
> Está documentado también lo que **no** hace: es un nodo único, el
> operador ve los saldos, y no lo ha auditado nadie. Esa parte ocupa tanto
> como la de los logros, a propósito.
>
> [enlace]

**Por qué funciona aquí**: sin jerga innecesaria, con una cifra memorable,
y la honestidad como rasgo del trabajo en vez de como disculpa.

---

## 3. Correo directo a alguien concreto

> Hola [nombre],
>
> Te escribo porque [motivo concreto: trabajas en X / escribiste sobre Y].
>
> He publicado un trabajo comparativo que quizá te resulte útil:
> implementé el mismo circuito de liquidación en cinco sistemas de prueba
> y medí las diferencias en condiciones idénticas. La parte que creo que
> aporta algo es que la comparación se hizo sobre una **aplicación
> completa**, no sobre circuitos de referencia — y varios de los hallazgos
> solo aparecen así. Por ejemplo, que AIR carezca de restricciones de
> copia no se descubre implementando SHA-256.
>
> No busco nada concreto; si te sirve, bien, y si ves algo mal, me
> interesa saberlo. El documento de auditoría incluye una sección con los
> puntos donde tengo menos confianza.
>
> [enlace]

**Reglas para este**: personalizar el motivo de verdad —un correo genérico
se nota—, no pedir nada, y ser breve. Si no cabe en la pantalla del móvil,
sobra.

---

## 4. Una línea, para cuando alguien pregunte

> El mismo circuito de liquidación implementado en cinco sistemas de
> prueba, medido, con los hallazgos y los límites documentados.

---

## Lo que NO conviene decir

**"Infraestructura soberana para un nuevo orden económico"** — un técnico
del sector deja de leer en el primer párrafo. Es la clase de frase que
hace que el trabajo bueno parezca poco serio.

**"Resuelve el problema de la privacidad en CBDC"** — no lo resuelve.
Demuestra que ciertas propiedades son construibles y mide lo que cuestan.
La diferencia importa y se nota.

**Omitir que es un nodo único** — se descubre en cinco minutos, y quien lo
descubra por su cuenta desconfiará también de lo que sí es cierto.

**Números sin contexto** — "verificar en 4 ms" invita a preguntar
"¿comparado con qué, en qué máquina, cuántas ejecuciones?". Darlo con su
matiz desde el principio evita esa conversación.

---

## Antes de publicar

- [ ] Decidir el nombre del repositorio.
- [ ] Comprobar que `README.md` es lo primero que se ve y que los enlaces
      internos funcionan.
- [ ] Revisar que no queden claves, rutas locales ni datos personales en
      el código y los ficheros de prueba.
- [ ] Ejecutar la suite completa desde un clon limpio.
- [ ] Fechar la publicación: si más adelante se envía a una convocatoria,
      tener la autoría datada ayuda.
