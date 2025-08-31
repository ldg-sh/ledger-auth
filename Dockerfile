FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
RUN cargo install cargo-chef
WORKDIR /ledger-auth

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /ledger-auth/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin ledger-auth

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /ledger-auth/target/release/ledger-auth /usr/local/bin/ledger-auth
CMD ["/usr/local/bin/ledger-auth"]