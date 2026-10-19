# Etapa 1: compilacion (ring necesita un compilador de C).
FROM rust:1.99-slim AS build
RUN apt-get update \
    && apt-get install -y --no-install-recommends gcc libc6-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
# Cachea las dependencias con un main vacio antes de copiar el codigo real.
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
    && cargo build --release \
    && rm -rf src
COPY src ./src
COPY templates ./templates
# COPY conserva los mtime del contexto: sin el touch, cargo reusaria el
# main de mentira de la cache y la imagen saldria con un binario vacio.
RUN find src -name '*.rs' -exec touch {} + \
    && cargo build --release

# Etapa 2: runtime minimo (las plantillas Askama van embebidas).
FROM debian:trixie-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /app/target/release/rustaurant /usr/local/bin/rustaurant
COPY static ./static
ENV HOST=0.0.0.0 PORT=3000
EXPOSE 3000
CMD ["rustaurant"]
