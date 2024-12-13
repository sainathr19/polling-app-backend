# Use a Rust base image with Cargo installed
FROM rust AS builder

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Now copy the source code
COPY ./src ./src

ARG DATABASE_URL
ARG JWT_SECRET

# Set environment variables
ENV DATABASE_URL=${DATABASE_URL}
ENV JWT_SECRET=${JWT_SECRET}

RUN touch .env
# Create .env file and write database URL and JWT secret
RUN echo "DATABASE_URL=${DATABASE_URL}" > .env && echo "JWT_SECRET=${JWT_SECRET}" >> .env

# apt install libssl-dev and openssl
RUN apt-get update && apt-get install -y libssl-dev openssl


RUN cat .env && echo "Environment variables set"

# Build your application
RUN cargo build --release

CMD [ "cargo","run","--release" ]