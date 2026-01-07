FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /ledger-auth

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY entity ./entity
COPY migration ./migration
COPY src ./src
COPY proto ./proto
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN apt-get update && apt-get install -y protobuf-compiler
ENV PROTOC=/usr/bin/protoc

COPY --from=planner /ledger-auth/recipe.json recipe.json

COPY Cargo.toml Cargo.lock ./
COPY entity ./entity
COPY migration ./migration
COPY src ./src
COPY proto ./proto

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin ledger-auth

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /ledger-auth/target/release/ledger-auth /usr/local/bin/ledger-auth
CMD ["/usr/local/bin/ledger-auth"]