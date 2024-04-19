# Use the bookworm Rust image as the base image
FROM rust:bookworm as builder

# Set working directory
WORKDIR /app

# Copy Cargo.toml and Cargo.lock to the working directory
COPY Cargo.toml Cargo.lock ./

# Create a dummy Rust project and download dependencies
RUN mkdir -p src tests \
     && echo "fn main() {}" > src/main.rs \
     && echo "fn main() {}" > tests/integration.rs \
     && cargo build --release \
     && rm -rf src tests

# Copy the project source code to the working directory
COPY . .

# Build project
RUN cargo build --release

# Use a smaller base image, such as debian:bookworm-slim
FROM debian:bookworm-slim

# Install deps
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the compilation stage
COPY --from=builder /app/target/release/bitseed /usr/local/bin/bitseed

# Set the default command of the container
ENTRYPOINT ["/usr/local/bin/bitseed"]
