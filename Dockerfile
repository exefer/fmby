FROM rust:alpine AS builder
WORKDIR /usr/src/fmby
COPY . .
RUN cargo build --release

FROM alpine:latest
COPY --from=builder /usr/src/fmby/target/release/fmby /usr/bin/fmby
CMD ["/usr/bin/fmby"]
