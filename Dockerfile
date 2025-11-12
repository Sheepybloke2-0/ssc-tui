FROM rust:1.83

# Install additional tools and dependencies
RUN apt-get update && apt-get install -y \
    git \
    vim \
    curl \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install Rust components
RUN rustup component add rustfmt clippy

# Set working directory
WORKDIR /workspace

# Copy project files
COPY . .

# Build dependencies cache (optional - uncomment when you have dependencies)
# RUN cargo build --release
# RUN rm -rf target/release/deps/ssc_tui*

# Default command
CMD ["/bin/bash"]
