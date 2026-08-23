FROM rust:1.97.1-bookworm AS build
WORKDIR /src
COPY . .
RUN cargo build --locked --release -p tepp_api --bin tepp-loopback

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/tepp-loopback /usr/local/bin/tepp-loopback
USER 65532:65532
ENTRYPOINT ["/usr/local/bin/tepp-loopback"]
