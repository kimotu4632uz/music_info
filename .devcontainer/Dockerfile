FROM rust:1.56.0-slim-bullseye
RUN apt update && \
    apt install -y git ssh pkg-config build-essential libclang-dev libtagc0-dev && \
    apt clean && \
    rm -rf /var/lib/apt/lists/*

