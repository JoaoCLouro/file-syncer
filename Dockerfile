FROM rust:slim-trixie AS server
ENV
WORKDIR fs
COPY Cargo.toml Cargo.lock ./
RUN cargo clean && cargo build --release && mkdir tmp_data/ && chown sever:server tmp_data
USER server
EXPOSE 8080
ENTRYPOINT ["target/release/file-syncer"]
CMD["server"]


