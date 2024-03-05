# Rust builder
FROM lukemathwalker/cargo-chef:0.1.63-rust-1.76.0-bullseye AS chef
WORKDIR /app
RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y musl-tools libssl-dev

# Plan build
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Build application
FROM chef AS builder
ARG MOLD_VERSION=2.4.0

# Install Mold
RUN wget https://github.com/rui314/mold/releases/download/v${MOLD_VERSION}/mold-${MOLD_VERSION}-x86_64-linux.tar.gz \
    && tar xf mold-${MOLD_VERSION}-x86_64-linux.tar.gz \
    && mv mold-${MOLD_VERSION}-x86_64-linux /opt/mold \
    && rm mold-${MOLD_VERSION}-x86_64-linux.tar.gz
ENV PATH=${PATH}:/opt/mold/bin

# Build dependencies - this is the caching Docker layer
COPY --from=planner /app/recipe.json recipe.json
RUN mold -run cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl --bin user-apt-get

# Build application
COPY . .
RUN RUSTFLAGS='-C target-feature=+crt-static' mold -run cargo build --release --bin user-apt-get --target x86_64-unknown-linux-musl
RUN mv /app/target/x86_64-unknown-linux-musl/release/user-apt-get /app/user-apt-get

FROM scratch AS final
COPY --from=builder /app/user-apt-get /user-apt-get
