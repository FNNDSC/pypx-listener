FROM rust:1.70-slim-bullseye as base
FROM base as builder
ARG CARGO_TERM_COLOR=always


# Build dependencies only to take advantage of layer caching
# https://github.com/rust-lang/cargo/issues/12370
RUN cargo new --bin /usr/local/src/rx-repack
WORKDIR /usr/local/src/rx-repack
COPY Cargo.toml Cargo.lock ./
# Build dependencies only
RUN cargo build --release

# Copy real sources and build actual probject
COPY src src
RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update \
    && apt-get install -y dcmtk \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir /var/received_dicoms \
    && chmod g+rwx /var/received_dicoms

COPY --from=builder /usr/local/src/rx-repack/target/release/rx-repack /usr/local/bin/rx-repack

EXPOSE 11113
CMD ["storescp", "--fork", "-od", "/var/received_dicoms", "-pm", "-sp", "-xcr", "px-recount --xcrdir '#p' --xcrfile '#f' --verbosity 0 --logdir /home/dicom/log --datadir /home/dicom/data --cleanup", "11113"]
