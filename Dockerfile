FROM rust:1.55 as build

RUN USER=root cargo new --bin todo_api
WORKDIR /todo_api
COPY ./Cargo.toml .
COPY ./Cargo.lock .
RUN cargo build --release
RUN rm src/*.rs


COPY ./migrations ./migrations
COPY ./src ./src
COPY ./diesel.toml .
RUN rm ./target/release/deps/todo_api*
RUN cargo build --release

FROM debian:buster-slim

RUN apt-get update && apt-get install libpq5 -y

EXPOSE 7000

COPY --from=build /todo_api/target/release/todo_api /bin
CMD ["todo_api"]
