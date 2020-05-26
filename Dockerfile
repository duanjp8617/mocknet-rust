FROM rust:1.43

WORKDIR /usr/src/myapp
COPY . .

RUN cargo install --path .

CMD ["mocknet-rust"]