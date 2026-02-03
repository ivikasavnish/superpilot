# Multi-stage build for minimal final image
FROM rust:1.93 as builder

WORKDIR /build

# Copy manifests
COPY Cargo.toml ./

# Copy source code
COPY src ./src

# Build for production (prod feature)
RUN cargo build --release --features prod

# Build for recorder (recorder feature) 
RUN cargo build --release --features recorder --bin superpilot && \
    mv target/release/superpilot target/release/superpilot-recorder

# Final stage - minimal runtime image
FROM debian:bookworm-slim

# Install CA certificates for TLS
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binaries from builder
COPY --from=builder /build/target/release/superpilot /app/superpilot-prod
COPY --from=builder /build/target/release/superpilot-recorder /app/superpilot-recorder

# Default to prod binary
COPY --from=builder /build/target/release/superpilot /app/superpilot

# Expose ports
EXPOSE 8080 9090

# Default command runs prod binary
CMD ["/app/superpilot"]
