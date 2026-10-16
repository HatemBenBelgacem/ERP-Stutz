# ---------- Build: Server-Binary und WebAssembly mit cargo-leptos ----------
FROM rust:1-bookworm AS builder

# Offizielles Node 22 für Tailwind und DaisyUI (das npm aus Debian
# installiert die Linux-Variante von Tailwind nicht zuverlässig)
COPY --from=node:22-bookworm-slim /usr/local/bin/node /usr/local/bin/node
COPY --from=node:22-bookworm-slim /usr/local/lib/node_modules /usr/local/lib/node_modules
RUN ln -s /usr/local/lib/node_modules/npm/bin/npm-cli.js /usr/local/bin/npm \
    && ln -s /usr/local/lib/node_modules/npm/bin/npx-cli.js /usr/local/bin/npx

RUN rustup target add wasm32-unknown-unknown \
    && curl -L --proto '=https' --tlsv1.2 -sSf \
       https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
    && cargo binstall cargo-leptos@0.3.10 -y

WORKDIR /app
# Ohne Lockfile installieren: das Lockfile vom Mac enthält nur die
# macOS-Variante von Tailwind
COPY package.json ./
RUN npm install --no-audit --no-fund \
    && ls node_modules/@tailwindcss | grep oxide-linux

COPY . .
RUN cargo leptos build --release

# ---------- Laufzeit: nur Binary und statische Dateien ----------
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/weiterbildung /app/weiterbildung
COPY --from=builder /app/target/site /app/site
COPY --from=builder /app/Cargo.toml /app/Cargo.toml

ENV LEPTOS_OUTPUT_NAME=weiterbildung \
    LEPTOS_SITE_ROOT=site \
    LEPTOS_SITE_PKG_DIR=pkg \
    LEPTOS_SITE_ADDR=0.0.0.0:8080 \
    LEPTOS_ENV=PROD \
    RUST_LOG=info

EXPOSE 8080
CMD ["/app/weiterbildung"]
