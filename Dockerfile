FROM lukemathwalker/cargo-chef:latest-rust-alpine AS chef
WORKDIR /usr/src/fmby

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /usr/src/fmby/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release

FROM alpine:latest
COPY --from=builder /usr/src/fmby/target/release/fmby /usr/bin/fmby
CMD ["/usr/bin/fmby"]
