# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM rust:1.54-buster as cargo-build

RUN apt-get update

WORKDIR /usr/src/oidc-token-test-service

COPY Cargo.toml Cargo.toml

RUN mkdir src/

RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs

COPY . .

RUN cargo build --release

# RUN cargo install --path .

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM debian:bullseye-slim

RUN apt-get update && rm -rf /var/lib/apt/lists/*

RUN addgroup --system -gid 1000 runtme

RUN adduser --system --disabled-login --shell /bin/sh -uid 1001 --ingroup runtme runtme

COPY --from=cargo-build /usr/src/oidc-token-test-service/target/release/oidc-token-test-service /usr/local/bin/oidc-token-test-service

COPY --from=cargo-build /usr/src/oidc-token-test-service/static/private_key.der /usr/local/etc/private_key.der

RUN chown runtme:runtme /usr/local/bin/oidc-token-test-service

USER runtme

ENV BIND="0.0.0.0"

ENV PORT="8080"

CMD ["sh", "-c", "oidc-token-test-service /usr/local/etc/private_key.der -p ${PORT} -b ${BIND}"]
