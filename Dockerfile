FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev pkgconf openssl-dev
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main(){}' > src/main.rs && cargo build --release && rm -rf src
COPY . .
RUN cargo build --release

FROM alpine:3.20
RUN apk add --no-cache ca-certificates tini wget
COPY --from=builder /build/target/release/nexus-gate /usr/local/bin/
COPY configs/default.toml /etc/nexus-gate/config.toml
COPY migrations/ /etc/nexus-gate/migrations/
EXPOSE 7900
ENTRYPOINT ["tini", "--"]
CMD ["nexus-gate", "--config", "/etc/nexus-gate/config.toml"]
