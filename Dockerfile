FROM ekidd/rust-musl-builder:1.51.0 as dependencies

WORKDIR /usr/src/try_gluon

USER root

RUN apt-get update && apt-get install -y curl gnupg make g++ git pkg-config libgnutls30
RUN curl -sL https://deb.nodesource.com/setup_16.x | bash - && \
    apt-get update && apt-get install -y nodejs

RUN curl -L https://github.com/rust-lang-nursery/mdBook/releases/download/v0.1.2/mdbook-v0.1.2-x86_64-unknown-linux-gnu.tar.gz | tar -xvz && \
    mv mdbook /usr/local/bin/

RUN rustup default 1.51.0 && rustup target add x86_64-unknown-linux-musl

COPY package.json package-lock.json ./
RUN npm ci

COPY ./scripts/setup_cache.sh .
ARG RUSTC_WRAPPER
ENV SCCACHE_REDIS=redis://localhost
RUN . ./setup_cache.sh
# Cache the built dependencies
COPY gluon_master/Cargo.toml gluon_master/
COPY gluon_crates_io/Cargo.toml gluon_crates_io/
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p gluon_master/src && touch gluon_master/src/lib.rs \
    && mkdir -p gluon_crates_io/src && touch gluon_crates_io/src/lib.rs \
    && mkdir -p src/app && echo "fn main() { }" > src/app/main.rs \
    && mkdir -p src/bin && echo "fn main() { }" > src/bin/generate_docs.rs \
    && mkdir -p tests && touch tests/run.rs \
    && echo "fn main() { }" > build.rs
ARG RELEASE=
ARG CARGO_INCREMENTAL=
RUN cargo build --target=x86_64-unknown-linux-musl ${RELEASE} --tests --bins --all-features

FROM dependencies as builder
ARG RELEASE=
ARG CARGO_INCREMENTAL=

COPY ./scripts/build_docs.sh ./scripts/
RUN ./scripts/build_docs.sh

COPY . .

RUN npx webpack-cli --mode=production

RUN touch gluon_master/src/lib.rs && \
    touch gluon_crates_io/src/lib.rs && \
    cargo build --target=x86_64-unknown-linux-musl ${RELEASE} --tests --bins --all-features
RUN cargo run --target=x86_64-unknown-linux-musl ${RELEASE} --all-features --bin generate_docs

FROM alpine:3.12

WORKDIR /root/

RUN apk add certbot openssl

RUN mkdir -p ./target/dist
COPY --from=builder /usr/src/try_gluon/target/x86_64-unknown-linux-musl/release/try_gluon .
COPY --from=builder /usr/src/try_gluon/target/dist ./target/dist
COPY --from=builder /usr/src/try_gluon/public/ ./public
COPY --from=builder /usr/src/try_gluon/src/ ./src
COPY --from=builder /usr/src/try_gluon/Cargo.lock .
COPY --from=builder /usr/src/try_gluon/src/robots.txt /usr/src/try_gluon/src/favicon.ico ./target/dist/

ENV RUST_BACKTRACE 1

EXPOSE 80
EXPOSE 443

CMD ./try_gluon --https
