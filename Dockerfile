FROM rust:1.97 as builder

WORKDIR /usr/src/app

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl3 \
    ca-certificates \
    openssh-client git \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p -m 0700 ~/.ssh && \
    ssh-keyscan git.kundeng.us >> ~/.ssh/known_hosts

COPY Cargo.toml Cargo.lock ./

RUN --mount=type=ssh mkdir src && \
    echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && \
    cargo build --release --quiet && \
    rm -rf src target/release/deps/songparser*

COPY src ./src
COPY .env ./.env

RUN --mount=type=ssh \
    cargo build --release --quiet

FROM debian:trixie-slim

RUN apt-get update && apt-get install -y ca-certificates libssl-dev libssl3 && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

COPY --from=builder /usr/src/app/target/release/songparser .

COPY --from=builder /usr/src/app/.env .

CMD ["./songparser"]
