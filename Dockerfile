FROM rust:1.97.1-bookworm@sha256:0e2bcaef56d041a486784e54104a81aebe0da44bd03019bd70bc0401e42e4a97 AS build
WORKDIR /src
COPY . .
RUN cargo build --locked --release -p tepp_api --bin tepp-loopback

FROM debian:bookworm-slim@sha256:abd67ffcfa541b485a3dff59865ab629aa048a6c613e639d36e7456b0b229241
RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/tepp-loopback /usr/local/bin/tepp-loopback
USER 65532:65532
HEALTHCHECK --interval=10s --timeout=3s --start-period=2s --retries=5 \
    CMD curl --fail --silent --show-error \
        --header "content-type: application/json" \
        --header "tepp-consumer: lineageweave" \
        --header "tepp-contract-version: 1" \
        --data '{"contract_version":1,"consumer_code":"lineageweave","knowledge_cutoff":"2026-08-20T00:00:00Z","subject_post_id":"health-post","events":[{"event_id":"health-event","source_post_id":"health-post","event_type_code":"health_probe","event_label":"Health probe","event_time":"2026-08-20T00:00:00Z","available_time":"2026-08-20T00:00:00Z","project_reference":null,"actor_references":["health-actor"]}]}' \
        http://127.0.0.1:18081/v1/temporal-context >/dev/null \
        || exit 1
ENTRYPOINT ["/usr/local/bin/tepp-loopback"]
