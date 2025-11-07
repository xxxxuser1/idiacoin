# Build stage
FROM rust:1.70 as builder

WORKDIR /usr/src/idia
COPY . .

RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install dependencies
RUN apt-get update && apt-get install -y \
    tor \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary and config
COPY --from=builder /usr/src/idia/target/release/idia-core /usr/local/bin/
COPY config.toml /etc/idia/

# Create idia user
RUN useradd -r -s /bin/false idia && \
    mkdir -p /var/lib/idia && \
    chown -R idia:idia /var/lib/idia /etc/idia

# Configure Tor
COPY tor/torrc /etc/tor/
RUN chown -R debian-tor:debian-tor /etc/tor

USER idia
VOLUME ["/var/lib/idia"]
EXPOSE 8080 8081

CMD ["idia-core", "--config", "/etc/idia/config.toml"]