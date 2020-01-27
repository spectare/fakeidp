# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM rust:latest as cargo-build

RUN apt-get update

RUN apt-get install musl-tools -y

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/oidc-token-test-service

COPY Cargo.toml Cargo.toml

RUN mkdir src/

RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs

RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl

RUN rm -f target/x86_64-unknown-linux-musl/release/deps/oidc*

COPY . .

RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl

# RUN cargo install --path .

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM alpine:latest

RUN addgroup -g 1000 runtme

RUN adduser -D -s /bin/sh -u 1001 -G runtme runtme

COPY --from=cargo-build /usr/src/oidc-token-test-service/target/x86_64-unknown-linux-musl/release/oidc-token-test-service /usr/local/bin/oidc-token-test-service

COPY --from=cargo-build /usr/src/oidc-token-test-service/static/private_key.der /usr/local/etc/private_key.der

RUN chown runtme:runtme /usr/local/bin/oidc-token-test-service

RUN chown runtme:runtme /usr/local/etc/private_key.der

USER runtme

ENV BIND="0.0.0.0"

ENV PORT="8080"

CMD ["sh", "-c", "oidc-token-test-service /usr/local/etc/private_key.der -p ${PORT} -b ${BIND}"]
