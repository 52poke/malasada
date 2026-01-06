# Use the official Rust image as a builder
FROM rust:1 AS builder

# Create a new empty shell project
WORKDIR /usr/src/malasada
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build only the dependencies to cache them
RUN cargo build --release

# Copy the actual source code
COPY src ./src

# Touch main.rs to ensure it's rebuilt
RUN touch src/main.rs

# Build the application
RUN cargo build --release

# Use a minimal base image for the runtime
FROM debian:bookworm-slim

# Install CA certificates for HTTPS (S3) and clean up
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder
COPY --from=builder /usr/src/malasada/target/release/malasada /usr/local/bin/malasada

# Expose the application port
ENV PORT=3000
EXPOSE 3000

# Set the entrypoint
CMD ["malasada"]
