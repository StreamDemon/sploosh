FROM rust:1.91-bookworm

WORKDIR /workspace

ENV CARGO_INCREMENTAL=0

RUN rustup component add rustfmt clippy

CMD ["bash"]
