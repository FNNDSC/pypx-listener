FROM docker.io/lukemathwalker/cargo-chef:0.1.61-rust-1.71-slim-bullseye AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update \
    && apt-get install -y dcmtk \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir /var/received_dicoms \
    && chmod g+rwx /var/received_dicoms

COPY --from=builder /app/target/release/rx-repack /usr/local/bin/rx-repack

EXPOSE 11113
CMD ["storescp", "--fork", "-od", "/var/received_dicoms", "-pm", "-sp", "-xcr", "px-recount --xcrdir '#p' --xcrfile '#f' --verbosity 0 --logdir /home/dicom/log --datadir /home/dicom/data --cleanup", "11113"]
