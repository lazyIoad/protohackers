FROM rust:1.66-slim-buster
WORKDIR /usr/src/protohackers
COPY . .
RUN cargo install --path .
CMD ["protohackers", "0.0.0.0"]

