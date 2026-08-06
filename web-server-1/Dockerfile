# Step 1: Build stage
FROM rust:1.85-slim as builder

WORKDIR /app

# Copy Cargo configuration files
COPY Cargo.toml Cargo.lock ./

# Copy source code and Askama HTML templates
COPY src ./src
COPY templates ./templates

# Build release binary
RUN cargo build --release

# Step 2: Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies (e.g. ca-certificates)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy compiled binary from builder
COPY --from=builder /app/target/release/web-server-1 /app/web-server-1

# Expose port 8080
EXPOSE 8080

# Configure environment variables
ENV SERVER_ADDR=0.0.0.0:8080

# Command to run the application
CMD ["/app/web-server-1"]
