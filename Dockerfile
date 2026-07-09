FROM rust:1.94.1 AS dependencies

WORKDIR /usr/src/try_gluon

USER root

RUN apt-get update && apt-get install -y curl gnupg make g++ git pkg-config libgnutls30
RUN curl -sL https://deb.nodesource.com/setup_20.x | bash - && \
    apt-get update && apt-get install -y nodejs

RUN curl -L https://github.com/rust-lang-nursery/mdBook/releases/download/v0.1.2/mdbook-v0.1.2-x86_64-unknown-linux-gnu.tar.gz | tar -xvz && \
    mv mdbook /usr/local/bin/

COPY package.json package-lock.json ./
RUN npm ci

FROM dependencies AS builder

# COPY target/x86_64-unknown-linux-musl/${RELEASE:-debug}/try_gluon .

COPY ./src ./src
COPY elm.json webpack.config.js ./
RUN npx webpack-cli --mode=production

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

ENV RUST_BACKTRACE=1

EXPOSE 80
EXPOSE 443

CMD ./try_gluon --https
