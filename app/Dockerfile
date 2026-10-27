# Replace this Dockerfile with one that builds and runs YOUR service.
#
# Your service should listen on the container port you map in docker-compose.yml
# (8000 is preconfigured as an example) and reach PostgreSQL via the DATABASE_URL
# environment variable. The XML exports are mounted read-only at /data.
#
# Typical starting points:
#
#   Python:       FROM python:3.12-slim
#   Kotlin/Java:  multi-stage build, e.g.
#                 FROM gradle:8-jdk21 AS build / FROM eclipse-temurin:21-jre
#
# The placeholder below only exists so `docker compose up` works out of the box.

  # --- build ---
  FROM rust:1.94-bookworm AS build
  WORKDIR /app
  # Cache dependencies: copy manifests first, build a dummy lib/bin, then real sources
  COPY Cargo.toml Cargo.lock ./
  RUN mkdir src && echo "fn main() {}" > src/main.rs \
      && cargo build --release \
      && rm -rf src
  COPY src ./src
  COPY migrations ./migrations
  COPY data ./data
  COPY .sqlx ./.sqlx
  ENV SQLX_OFFLINE=true
  RUN touch src/main.rs && cargo build --release
  # --- runtime ---
  FROM debian:bookworm-slim
  RUN apt-get update \
      && apt-get install -y --no-install-recommends ca-certificates \
      && rm -rf /var/lib/apt/lists/*
  WORKDIR /app
  COPY --from=build /app/target/release/rust-claims-api /usr/local/bin/rust-claims-api
  EXPOSE 8000
  ENV RUST_LOG=info
  CMD ["rust-claims-api"]
