FROM rust AS builder

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./

COPY ./src ./src

ARG DATABASE_URL
ARG JWT_SECRET

ENV DATABASE_URL=${DATABASE_URL}
ENV JWT_SECRET=${JWT_SECRET}

RUN touch .env

RUN echo "DATABASE_URL=${DATABASE_URL}" > .env && echo "JWT_SECRET=${JWT_SECRET}" >> .env

RUN apt-get update && apt-get install -y libssl-dev openssl


RUN cat .env

RUN cargo build --release

CMD [ "cargo","run","--release" ]