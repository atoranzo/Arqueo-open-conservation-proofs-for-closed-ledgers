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

## Cómo se usa, exactamente

El método no ha cambiado desde el principio, y es el que explica todo lo demás:

1. Se **mide** primero. Una lectura pura sobre el árbol produce los números; un
   resumen recordado no es fuente.
2. Se discute sobre esos números, y **el autor decide**.
3. El asistente propone un **bloque**: un guion que aplica el cambio y que trae
   sus propias compuertas, incluido su camino rojo.
4. **El autor lo ejecuta en su máquina**, corre el canon —`tools/canon.sh
   --sello`, que ejecuta la suite de cada crate en release y las ocho
   herramientas de `tools/`— y **commitea sólo lo que sale verde**.

De ese método se siguen dos cosas que este documento afirma y que el registro
sostiene:

- **Nada entra sin pasar las compuertas.** Lo que se propone y el canon rechaza
  no llega a `main`. Los bloques que murieron en su puerta están contados en los
  asientos, con la razón por la que murieron.
- **Ninguna afirmación de este repositorio descansa en el conocimiento del
  modelo.** Lo que se afirma del presente está verificado contra el árbol; lo
  que no se ha medido se declara como no medido, y lo que se midió y salió falso
  queda escrito como falso.

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
especificación**. No hay ninguna parte de este repositorio que sea salida de un
modelo sin medición, ejecución y aceptación del autor.

## Autoría y responsabilidad

El autor de Arqueo es **Ángel José Toranzo Portela**, y es el único. Un
asistente no figura como autor ni como coautor: en la Unión Europea, lo generado
por una máquina sin aportación intelectual humana sustancial no da lugar a
derechos de autor, y aquí la decisión, la medición y la aceptación son del
autor. Quien quiera discutir una decisión de diseño o de código tiene enfrente a
una persona que la explica.

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

## How it is used, exactly

The method has not changed since the beginning, and it explains everything else:

1. **Measure first.** A pure reading over the tree produces the numbers; a
   remembered summary is not a source.
2. Those numbers are discussed, and **the author decides**.
3. The assistant proposes a **block**: a script that applies the change and
   brings its own gates, its red path included.
4. **The author runs it on his own machine**, runs the canon —`tools/canon.sh
   --sello`, which runs every crate's suite in release plus the eight tools
   under `tools/`— and **commits only what comes out green**.

Two things follow from that method, and the record sustains them:

- **Nothing gets in without passing the gates.** What is proposed and the canon
  rejects never reaches `main`. The blocks that died at their own gate are
  counted in the entries, with the reason they died.
- **No claim in this repository rests on the model's knowledge.** What is
  claimed about the present is verified against the tree; what has not been
  measured is declared as not measured, and what was measured and came out false
  is written down as false.

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
specification**. No part of this repository is a model's output without the
author's measurement, execution and acceptance.

## Authorship and accountability

The author of Arqueo is **Ángel José Toranzo Portela**, and he is the only one.
An assistant is not listed as an author or a co-author: in the European Union,
what a machine generates without substantial human intellectual contribution
does not give rise to copyright, and here the decision, the measurement and the
acceptance are the author's. Anyone who wants to argue about a design or code
decision has a person in front of them who explains it.
