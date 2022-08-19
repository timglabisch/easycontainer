FROM ubuntu:22.10
COPY ./easycontainer_lib /usr/src/easycontainer_lib
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.62.1

RUN apt-get update -y \
  && apt-get install -y build-essential curl \
  && bash -c "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > /rustup.sh && chmod +x rustup.sh && /rustup.sh -y" \
  && rustup target add x86_64-unknown-linux-musl \
  && rustup target add aarch64-unknown-linux-musl

