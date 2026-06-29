FROM rust:1.91-bookworm

WORKDIR /workspace

ENV CARGO_INCREMENTAL=0
ENV PATH="/usr/local/cargo/bin:${PATH}"

RUN rustup component add rustfmt clippy

CMD ["bash"]
