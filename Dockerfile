## Build Stage ##
FROM alpine:latest AS builder

RUN apk add --no-cache git cargo

RUN git clone https://github.com/patrickramp/signing-service

WORKDIR /signing-service/

RUN cargo build --release

## Run Stage ##
FROM alpine:latest

RUN apk upgrade --no-cache \
    && apk add --no-cache libgcc

COPY --from=builder /actix-webserver/target/release/signing-service /usr/local/bin/signing-service

RUN adduser -H -S -s /sbin/nologin signer \
    && addgroup -S signer \
    && chown -R signer:signer /usr/local/bin/signing-service
    && chown -R signer:signer /certs/

COPY ./public /var/www/public

USER signer

ENTRYPOINT [ "/usr/local/bin/signing-service" ]
