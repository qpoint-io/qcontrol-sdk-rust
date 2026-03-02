FROM us-docker.pkg.dev/qpoint-edge/public/qcontrol:latest AS qcontrol

FROM rust:1-bookworm

# Install additional dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    git \
    curl \
    ca-certificates \
    clang \
    libclang-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy qcontrol binary from upstream image
COPY --from=qcontrol /usr/local/bin/qcontrol /usr/local/bin/qcontrol

# Copy SDK source
WORKDIR /workspace
COPY . .

# Build examples to verify everything works, then clean
RUN cd examples && make clean && make all && make clean

# Add MOTD
COPY examples/motd.txt /etc/motd
RUN echo 'cat /etc/motd' >> /root/.bashrc

CMD ["/bin/bash"]
