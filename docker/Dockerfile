FROM rust:1.87-slim-bookworm AS builder
WORKDIR /app

COPY . .

# TODO: We should have an install script software in Ere, so we can avoid this step
# Install SP-1 toolchain and C/C++ build tools
RUN set -eux; \
    apt-get update && apt-get install -y --no-install-recommends \
    curl \
    ca-certificates \
    git \
    build-essential; \
    rm -rf /var/lib/apt/lists/*; \
    curl -L https://sp1up.succinct.xyz | bash; \
    ~/.sp1/bin/sp1up; \
    echo 'export PATH="$HOME/.sp1/bin:$PATH"' >> /etc/profile.d/sp1.sh
ENV PATH="/root/.sp1/bin:${PATH}"

# Build the application for release
RUN cargo build --release

# ---- Runtime Stage ----
FROM debian:bookworm-slim
WORKDIR /app

# Copy the binary and programs
COPY --from=builder /app/target/release/poost /app/poost
# TODO: We can include the `programs` directory along with the binary
# Copy the programs folder since we need those at runtime
COPY programs/ /app/programs/

# Expose port 3000 to the outside world (This is the port that the rust code currently listens on)
EXPOSE 3000
CMD ["/app/poost"]
