# syntax=docker/dockerfile:1

FROM chainguard/rust:latest-dev as build
USER root

WORKDIR /work
RUN apk add --no-cache openssl-dev

USER nonroot

COPY --chown=nonroot:nonroot . .
RUN cargo build --locked --release

FROM chainguard/glibc-dynamic:latest-dev
USER root

WORKDIR /tmp
RUN chown -R nonroot:nonroot /tmp
RUN apk add --no-cache openssl-dev

USER nonroot

COPY --from=build --chown=nonroot:nonroot /work/target/release/pyrite /usr/local/bin/pyrite

ENTRYPOINT ["/usr/local/bin/pyrite"]