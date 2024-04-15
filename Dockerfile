FROM rust:latest

RUN mkdir -p /usr/src/app
WORKDIR /usr/src/app

COPY . /usr/src/app

RUN cargo install --path /usr/src/app

ENTRYPOINT ["minasan"]
