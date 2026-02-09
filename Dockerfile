# syntax=docker/dockerfile:1
ARG SUPABASE_URL
ARG SUPABASE_API_KEY

FROM chainguard/rust:latest-dev as build
USER root

WORKDIR /work
RUN apk add --no-cache openssl-dev

USER nonroot

COPY --chown=nonroot:nonroot . .
RUN cargo build --locked --release

FROM chainguard/glibc-dynamic:latest-dev
ARG SUPABASE_URL
ARG SUPABASE_API_KEY

ENV SUPABASE_URL=$SUPABASE_URL
ENV SUPABASE_API_KEY=$SUPABASE_API_KEY

USER root

WORKDIR /tmp
RUN chown -R nonroot:nonroot /tmp
RUN apk add --no-cache openssl-dev

USER nonroot

COPY --from=build --chown=nonroot:nonroot /work/target/release/pyrite /usr/local/bin/pyrite

ENTRYPOINT ["/usr/local/bin/pyrite"]