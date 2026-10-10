# Arqueo para el OSS Scanner de Anthropic (`anthropics/oss-scanner`, `projects/arqueo`),
# desde el §800 de `AUDITORIA.md`.
#
# El escaner construye este fichero con el clon del repositorio como contexto y CON red,
# en una maquina de 16 nucleos, con 45 minutos desde el clon hasta la imagen terminada.
# Despues le quita la red y la analiza en /src con 2 nucleos y 8 GB: todo lo que un test
# necesite se baja aqui, y lo caro se compila aqui y no alli.
#
# ⚠️ Lo que se ha medido de este fichero y lo que no lo dice el §800: sus pasos se
# corrieron fuera de Docker, con el mismo `rustc` y en otra maquina; la imagen no se ha
# construido entera.

# El `rustc` con el que se mide el canon hoy (1.97.0), sobre Debian bookworm, la base de la
# plantilla del escaner. Clavado a la version, no al digest.
FROM rust:1.97.0-bookworm

ENV DEBIAN_FRONTEND=noninteractive
# `git` para los guiones de `tools/` que lo leen; `python3` para los `check_*.py` y la
# segunda implementacion de `tools/segunda/` (solo la biblioteca estandar).
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates git python3 \
    && rm -rf /var/lib/apt/lists/*

# Contrato del escaner: el clon vive en /src.
COPY . /src
WORKDIR /src

# Release, que es como corre el canon (`cargo test --release --locked`) y como se prueba
# en tiempo razonable, con informacion de depuracion, como pide la plantilla. Va en ENV y no
# en el RUN: si el agente lanza `cargo test --release`, reutiliza lo compilado aqui en vez de
# recompilar el workspace con 2 nucleos.
# ⚠️ Sin `-C force-frame-pointers=yes`, que la plantilla tambien pide: reserva `rbp`, y el
# ensamblador de `halo2curves`, que entra por `nova-snark` en `nova-experiment`, lo necesita.
# Con el flag no compila: «inline assembly requires more registers than available» (§800).
ENV CARGO_PROFILE_RELEASE_DEBUG=true

# Las dependencias del lock, de todas las plataformas: sin red, `cargo` ya no baja nada.
RUN cargo fetch --locked

# El workspace entero y los binarios de sus tests: lo que el canon corre, ya compilado.
RUN cargo build --release --locked --workspace \
    && cargo test --release --locked --workspace --no-run

# Desde aqui no hay red: que `cargo` no la busque.
ENV CARGO_NET_OFFLINE=true

# Los tests baratos, los del kit verificador y los suyos, para saber que la imagen funciona.
# Un test rojo no para la construccion: lo avisa.
RUN for c in zk-ssl-verify zk-ssl-wire zk-ssl-hash zk-ssl-air zk-ssl-medio zk-ssl-guardian; do \
      cargo test --release --locked -p "$c" || echo "AVISO: los tests de $c no salieron verdes"; \
    done
