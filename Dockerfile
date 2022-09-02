FROM ubuntu:22.04
COPY ./easycontainer_lib /usr/src/easycontainer_lib
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.62.1

# rust
RUN apt-get update -y \
  && apt-get install -y build-essential curl lsb-release \
  && bash -c "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > /rustup.sh && chmod +x rustup.sh && /rustup.sh -y" \
  && rustup target add x86_64-unknown-linux-musl \
  && rustup target add aarch64-unknown-linux-musl

# docker cli
RUN mkdir -p /etc/apt/keyrings && \
    bash -c "curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg" && \
    bash -c 'echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null' && \
    apt-get update && \
    apt-get install -y docker-ce-cli docker-compose-plugin

# easycontainer
COPY easycontainer /easycontainer
RUN cargo install --path /easycontainer

CMD [""]