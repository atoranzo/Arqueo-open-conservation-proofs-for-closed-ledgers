# Uso de IA generativa en Arqueo

> **English:** the same statement, section by section, further down:
> [Generative AI in Arqueo](#generative-ai-in-arqueo).

Este documento declara, para quien use o revise este repositorio, en qué medida
se ha usado un asistente de IA generativa para producirlo, y con qué método. No
es una nota al pie: el historial, la especificación y el registro de este
repositorio son tan documento del proyecto como el código, y quien los lea tiene
derecho a saber cómo se produjeron.

## Qué se usa

- **Modelo:** Claude, de Anthropic, a través de `claude.ai`. La versión concreta
  ha cambiado a lo largo del proyecto.
- **Desde cuándo:** desde la primera sesión del proyecto, el 29 de julio de 2026.
- **Dónde:** en conversaciones de `claude.ai` y en sesiones de Claude Code en la
  nube; el primer asiento de [`AUDITORIA.md`](./AUDITORIA.md) que declara una
  es el §589. Cada sesión trabaja en su propio contenedor, sobre una rama
  `claude/…` de este repositorio, y puede lanzar otras sesiones o agentes que
  escriben, revisan o comprueban por ella.

## Cómo se usa, exactamente

El método tiene cuatro pasos, y es el que explica todo lo demás. Lo que no ha
sido igual desde el principio es quién da el último: la sección «Fuera del paso
4», más abajo, dice cómo y desde cuándo.

1. Se **mide** primero. Una lectura pura sobre el árbol produce los números; un
   resumen recordado no es fuente.
2. Se discute sobre esos números, y **el autor decide**.
3. El asistente propone un **bloque**: un guion que aplica el cambio y que trae
   sus propias compuertas, incluido su camino rojo.
4. **El autor lo ejecuta en su máquina**, corre el canon —`tools/canon.sh
   --sello`, que ejecuta la suite de cada crate en release, las nueve
   herramientas de su bucle de `tools/` y el resto de compuertas que lista— y
   **commitea sólo lo que sale verde**.

De ese método se siguen dos cosas que este documento afirma y que el registro
sostiene:

- **Nada entra sin pasar las compuertas que declara su asiento.** En el paso 4
  son el canon entero, en la máquina del autor; en una sesión, las que corrió la
  sesión, y cuando no fueron el canon entero o el canon no salió verde, el
  asiento dice cuáles fueron y qué dieron. Los bloques que murieron en su puerta
  están contados en los asientos, con la razón por la que murieron.
- **Ninguna afirmación de este repositorio descansa en el conocimiento del
  modelo.** Lo que se afirma del presente está verificado contra el árbol; lo
  que no se ha medido se declara como no medido, y lo que se midió y salió falso
  queda escrito como falso.

### Fuera del paso 4: la sesión de Claude Code

Una sesión de Claude Code no propone el bloque para que lo ejecute el autor: lo
ejecuta ella. Eso empezó antes de que un asiento lo declarara. El S563-C
original (`3332c91`, 26 de septiembre de 2026), el commit más antiguo con la
marca de una sesión que cuenta el registro, llevaba `Co-Authored-By` y
`Claude-Session:` (§628); el §589 declara la primera sesión que trabaja en un
contenedor remoto, y ya pone la aceptación en la fusión del autor; y el §629
llama a la sesión que lo commitea «la del S619 al S628». El primer asiento que
dice en su apertura que su commit lo hizo una sesión, y no el autor en su
máquina, es el §628, y desde él lo dicen todos: del §628 al §699 son 71, y el
§700, que escribe esta sección, los cuenta. Los pasos 1 a 3 no cambian, y el
asiento de cada cambio que sale así lo declara en su apertura, casi siempre con
las mismas palabras: «fuera del paso 4 de `GENAI.md`». Del paso 4 cambian tres
cosas:

- **Quién aplica, comprueba y commitea.** La sesión, en su contenedor y sobre su
  rama, con la identidad del autor y sin líneas que la nombren, como pide
  [`CLAUDE.md`](./CLAUDE.md). Si otra sesión escribió o comprobó el cambio por
  ella, el asiento dice cuál hizo qué.
- **Sus compuertas.** Son las del canon, pero las corre la sesión en su
  contenedor, no el autor en su máquina, y nadie más las ve correr: lo que queda
  es lo que declara el asiento. Dice si el canon `--sello` salió VERDE sobre el
  árbol del cambio y, si no se corrió entero o no salió verde, qué compuertas se
  corrieron y qué dieron. Ninguna compuerta comprueba que lo declarado se
  corriera.
- **Quién acepta.** El autor, y su aceptación ya no es el commit, que no hace
  él, sino la entrada del cambio en `main`, que es suya. Hasta entonces el
  cambio vive en la rama de la sesión. El autor puede volver a correr el canon
  en su máquina antes de llevarlo a `main`, y algunos asientos dicen que lo
  hizo; ninguna regla lo exige.

## Dónde está el registro

**El marcado por commit no ha sido el método de este proyecto, y no se presenta
como tal.** El registro por cambio es [`AUDITORIA.md`](./AUDITORIA.md): un
asiento por cambio verificado, con su commit, sus contadores, lo que el cambio
**no** cierra y las lecciones que dejó. Es sustancialmente más detallado que un
mensaje de commit, y es el sitio donde mirar.

Además, cada RFC de [`spec/rfc/`](./spec/rfc/) declara en su cabecera, junto al
autor, las sesiones de trabajo en las que se redactó.

## Alcance

Se ha usado como asistencia en **código, prosa y redacción de la
especificación**. Lo que entra en `main` pasa por el método de arriba, y entrar
es la aceptación del autor. Hay una excepción, declarada donde ocurre: lo que el
asistente redactó en una sesión y entró con la aceptación pendiente lo dice el
propio documento, casi siempre con las palabras «integrar no es aceptar», y no
cuenta como decisión del autor hasta que un asiento suyo lo acepte. A 8 de
octubre de 2026 lo dicen `doc/ecst/`, `doc/firma-corta-evaluacion.md` y
`tools/segunda/`. `doc/integracion-vertical-evaluacion.md` y
`doc/blueprint-v2.md` lo decían hasta el §705, en el que el autor acepta el
primero con correcciones y el segundo en parte.

## Autoría y responsabilidad

El autor de Arqueo es **Angel Toranzo Portela**, y es el único. Es el nombre de
`NOTICE`, de `Cargo.toml` y de los commits; `CITATION.cff` y los preprints lo
escriben Angel Jose Toranzo Portela, y las cabeceras de los RFC, Ángel José
Toranzo Portela: es la misma persona. Un asistente no figura como autor ni como
coautor: en la Unión Europea, lo generado por una máquina sin aportación
intelectual humana sustancial no da lugar a derechos de autor, y aquí la
decisión, la medición y la aceptación son del autor. Quien quiera discutir una
decisión de diseño o de código tiene enfrente a una persona que la explica.

El asistente tampoco figura en los commits. Cuando es una sesión de Claude Code
la que commitea, y no el autor como dice el paso 4, [`CLAUDE.md`](./CLAUDE.md) le
indica que lo haga con la identidad del autor, sin líneas que nombren al
asistente, como `Co-Authored-By`, y que el asiento lo declare;
`.claude/settings.json` quita las que Claude Code añade por defecto. Ninguna
compuerta lo comprueba.

---

# Generative AI in Arqueo

This document states, for anyone using or reviewing this repository, to what
extent a generative AI assistant was used to produce it, and by what method. It
is not a footnote: the history, the specification and the record of this
repository are as much the project's documentation as the code is, and whoever
reads them is entitled to know how they were produced.

## What is used

- **Model:** Anthropic's Claude, through `claude.ai`. The specific version has
  changed over the course of the project.
- **Since when:** since the project's first session, on 29 July 2026.
- **Where:** in `claude.ai` conversations and in Claude Code sessions in the
  cloud; the first entry of [`AUDITORIA.md`](./AUDITORIA.md) that declares one
  is §589. Each session works in its own container, on a `claude/…` branch of
  this repository, and may launch other sessions or agents that write, review
  or check on its behalf.

## How it is used, exactly

The method has four steps, and it explains everything else. What has not been
the same since the beginning is who takes the last one: the section "Outside
step 4", further down, says how and since when.

1. **Measure first.** A pure reading over the tree produces the numbers; a
   remembered summary is not a source.
2. Those numbers are discussed, and **the author decides**.
3. The assistant proposes a **block**: a script that applies the change and
   brings its own gates, its red path included.
4. **The author runs it on his own machine**, runs the canon —`tools/canon.sh
   --sello`, which runs every crate's suite in release, the nine tools of its
   loop under `tools/` and the rest of the gates it lists— and **commits only
   what comes out green**.

Two things follow from that method, and the record sustains them:

- **Nothing gets in without passing the gates its entry declares.** At step 4
  they are the whole canon, on the author's machine; in a session, the ones the
  session ran, and when they were not the whole canon or the canon did not come
  out green, the entry says which they were and what they gave. The blocks that
  died at their own gate are counted in the entries, with the reason they died.
- **No claim in this repository rests on the model's knowledge.** What is
  claimed about the present is verified against the tree; what has not been
  measured is declared as not measured, and what was measured and came out false
  is written down as false.

### Outside step 4: the Claude Code session

A Claude Code session does not propose the block for the author to run: it runs
it itself. That began before any entry declared it. The original S563-C
(`3332c91`, 26 September 2026), the oldest commit bearing a session's mark that
the record counts, carried `Co-Authored-By` and `Claude-Session:` (§628); §589
declares the first session that works in a remote container, and already
places acceptance in the author's merge; and §629 calls the session that
commits it "la del S619 al S628" (the one of S619 to S628). The first entry
whose opening says its commit was made by a session, not by the author on his
machine, is §628, and every one since says so: from §628 to §699 there are 71,
and §700, which writes this section, counts them. Steps 1 to 3 do not change,
and the entry of every change made this way declares it in its opening, almost
always in the same words: "fuera del paso 4 de `GENAI.md`" (outside step 4 of
`GENAI.md`). Three things change in step 4:

- **Who applies, checks and commits.** The session, in its container and on its
  branch, under the author's identity and without lines that name it, as
  [`CLAUDE.md`](./CLAUDE.md) asks. If another session wrote or checked the
  change on its behalf, the entry says which did what.
- **Its gates.** They are the canon's, but the session runs them in its
  container, not the author on his machine, and nobody else sees them run: what
  remains is what the entry declares. It says whether the canon `--sello` came
  out GREEN on the change's tree and, if it was not run whole or did not come
  out green, which gates were run and what they gave. No gate checks that what
  is declared was run.
- **Who accepts.** The author, and his acceptance is no longer the commit, which
  he does not make, but the change's entry into `main`, which is his. Until
  then the change lives on the session's branch. The author may run the canon
  again on his machine before taking it to `main`, and some entries say he did;
  no rule requires it.

## Where the record is

**Per-commit marking has not been this project's method, and is not presented as
such.** The per-change record is [`AUDITORIA.md`](./AUDITORIA.md): one entry per
verified change, with its commit, its counters, what the change does **not**
close and the lessons it left. It is substantially more detailed than a commit
message, and it is the place to look.

Each RFC under [`spec/rfc/`](./spec/rfc/) also states in its header, next to the
author, the working sessions in which it was drafted.

## Scope

It has been used as assistance for **code, prose and the drafting of the
specification**. What gets into `main` goes through the method above, and
getting in is the author's acceptance. There is one exception, declared where it
happens: what the assistant drafted in a session and got in with acceptance
pending says so in the document itself, almost always in the words "integrar
no es aceptar" (integrating is not accepting), and does not count as the
author's decision until an entry of his accepts it. As of 8 October 2026,
`doc/ecst/`, `doc/firma-corta-evaluacion.md` and `tools/segunda/` say so.
`doc/integracion-vertical-evaluacion.md` and `doc/blueprint-v2.md` said so
until §705, in which the author accepts the first with corrections and the
second in part.

## Authorship and accountability

The author of Arqueo is **Angel Toranzo Portela**, and he is the only one. That
is the name in `NOTICE`, in `Cargo.toml` and in the commits; `CITATION.cff` and
the preprints write it Angel Jose Toranzo Portela, and the RFC headers, Ángel
José Toranzo Portela: it is the same person. An assistant is not listed as an
author or a co-author: in the European Union, what a machine generates without
substantial human intellectual contribution does not give rise to copyright,
and here the decision, the measurement and the acceptance are the author's.
Anyone who wants to argue about a design or code decision has a person in front
of them who explains it.

Nor is the assistant listed in the commits. When a Claude Code session makes
the commit, rather than the author as step 4 says, [`CLAUDE.md`](./CLAUDE.md)
tells it to do so under the author's identity, without lines that name the
assistant, such as `Co-Authored-By`, and to declare it in the entry;
`.claude/settings.json` removes the ones Claude Code adds by default. No gate
checks it.
