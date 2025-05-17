# ---- Build Stage ----
# Use the latest stable Rust image to ensure best support for Edition 2024
FROM rust:1.87-slim-bookworm AS builder

# Set the working directory in the container
WORKDIR /app

# Copy the current directory contents (project root) into the container at /app
# This works because docker-compose context is set to '..' (project root)
COPY . .

# TODO: We should have an install script software in Ere, so we can avoid this step
# Install the SP-1 toolchain (zkVM)
#  • curl → download the bootstrap script
#  • add ~/.sp1/bin to PATH so subsequent RUN layers see the binaries
RUN set -eux; \
    apt-get update && apt-get install -y --no-install-recommends curl ca-certificates; \
    curl -L https://sp1up.succinct.xyz | bash; \
    ~/.sp1/bin/sp1up; \
    echo 'export PATH="$HOME/.sp1/bin:$PATH"' >> /etc/profile.d/sp1.sh
ENV PATH="/root/.sp1/bin:${PATH}"

# Build the application for release
RUN cargo build --release

# ---- Runtime Stage ----
# Use a slim Debian image for a smaller final image
FROM debian:slim-bookwork

# Set the working directory in the container
WORKDIR /app

# Copy the compiled binary from the build stage
COPY --from=builder /app/target/release/poost /app/poost

# TODO: We can include the `programs` directory along with the binary
# Copy the programs folder since we need those at runtime
COPY programs/ /app/programs/

# Expose port 3000 to the outside world (This is the port that the rust code currently listens on)
EXPOSE 3000

# Command to run the application
CMD ["/app/poost"] 