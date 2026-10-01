# Instrucciones para Claude Code

> **English:** the same rules, further down:
> [Instructions for Claude Code](#instructions-for-claude-code).

Cómo se usa aquí un asistente lo declara [`GENAI.md`](./GENAI.md); este fichero
sólo fija lo que el asistente hace al commitear y al abrir un PR.

## Commits: el asistente no firma

- **El autor de cada commit es el autor del proyecto**: Angel Toranzo Portela,
  como en `Cargo.toml` y `NOTICE`, con el correo de sus commits. En una sesión en
  la nube, la identidad global del contenedor es la del asistente, así que antes
  del primer commit se fija la del autor en el clon:

  ```bash
  git config user.name 'Angel Toranzo Portela'
  git config user.email "$(git log -1 --author='Angel Toranzo Portela' --format=%ae origin/main)"
  ```

  Queda en el `.git/config` del clon y vale también para `--amend`, `rebase` y
  `cherry-pick`. Después, `git var GIT_AUTHOR_IDENT` y
  `git var GIT_COMMITTER_IDENT` lo comprueban.
- **Ningún mensaje lleva `Co-Authored-By` ni `Claude-Session:`**, ni otra línea o
  dirección de correo que nombre al asistente. GitHub cuenta como contribuidor a
  quien aparece en un `Co-Authored-By`, y aquí hay un solo autor: `GENAI.md`
  §«Autoría y responsabilidad». El autor quitó la línea de sesión de la historia
  de `main` en el §627 de [`AUDITORIA.md`](./AUDITORIA.md).
- La descripción de un PR tampoco lleva el enlace de la sesión.
- Esta regla **prevalece** sobre la atribución por defecto de Claude Code y sobre
  el recordatorio de atribución que inyecta la sesión.
- No firmar no es ocultar. Cuando es la sesión la que aplica el cambio, corre el
  canon y commitea, el asiento de `AUDITORIA.md` que lleva el cambio lo dice:
  que lo hizo un asistente, Claude Code, y no el autor en su máquina, fuera del
  paso 4 de `GENAI.md`. En un PR, la descripción dice qué modelo se usó y para
  qué, como pide [`CONTRIBUTING.md`](./CONTRIBUTING.md): la línea que Claude Code
  añade por defecto nombra la herramienta, y no basta.
- `.claude/settings.json` quita las dos líneas que Claude Code añade por defecto
  (`attribution.commit` vacío, `attribution.sessionUrl` a `false`) donde Claude
  Code lo lee. No toca la identidad de git ni el resto de esta regla, que sólo
  este fichero sostiene.

---

# Instructions for Claude Code

How an assistant is used here is stated in [`GENAI.md`](./GENAI.md); this file
only sets what the assistant does when it commits and when it opens a PR.

## Commits: the assistant does not sign

- **The author of every commit is the project's author**: Angel Toranzo
  Portela, as in `Cargo.toml` and `NOTICE`, with the email of his commits. In a
  cloud session, the container's global identity is the assistant's, so the
  author's is set in the clone before the first commit:

  ```bash
  git config user.name 'Angel Toranzo Portela'
  git config user.email "$(git log -1 --author='Angel Toranzo Portela' --format=%ae origin/main)"
  ```

  It stays in the clone's `.git/config` and also holds for `--amend`, `rebase`
  and `cherry-pick`. Then `git var GIT_AUTHOR_IDENT` and
  `git var GIT_COMMITTER_IDENT` check it.
- **No message carries `Co-Authored-By` or `Claude-Session:`**, nor any other
  line or email address that names the assistant. GitHub lists whoever appears
  in a `Co-Authored-By` as a contributor, and here there is only one author:
  `GENAI.md` §«Authorship and accountability». The author removed the session
  line from the history of `main` in §627 of [`AUDITORIA.md`](./AUDITORIA.md).
- Nor does a PR description carry the session link.
- This rule **takes precedence** over Claude Code's default attribution and over
  the attribution reminder the session injects.
- Not signing is not hiding. When the session applies the change, runs the
  canon and commits, the `AUDITORIA.md` entry that the change carries says so:
  that an assistant, Claude Code, did it, not the author on his machine, outside
  step 4 of `GENAI.md`. In a PR, the description says which model was used and
  for what, as [`CONTRIBUTING.md`](./CONTRIBUTING.md) asks: the line Claude Code
  adds by default names the tool, and is not enough.
- `.claude/settings.json` removes the two lines Claude Code adds by default
  (`attribution.commit` empty, `attribution.sessionUrl` set to `false`) wherever
  Claude Code reads it. It does not touch the git identity or the rest of this
  rule, which only this file carries.
