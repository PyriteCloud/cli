# syntax=docker/dockerfile:1
ARG SUPABASE_URL
ARG SUPABASE_API_KEY

FROM chainguard/rust:latest-dev as chef
USER root
RUN apk add --no-cache openssl-dev
RUN cargo install cargo-chef

WORKDIR /work

FROM chef AS planner
COPY --chown=nonroot:nonroot . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner --chown=nonroot:nonroot /work/client-rs ./client-rs
COPY --from=planner --chown=nonroot:nonroot /work/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
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

COPY --from=builder --chown=nonroot:nonroot /work/target/release/pyrite /usr/local/bin/pyrite

ENTRYPOINT ["/usr/local/bin/pyrite"]