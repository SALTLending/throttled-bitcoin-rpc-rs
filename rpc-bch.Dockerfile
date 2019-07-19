FROM rust
RUN apt update; apt install -y wait-for-it
RUN cargo install cargo-watch

# Get the rust up with the package, to have that cached
WORKDIR /app/src
RUN USER=root cargo new --lib rs
WORKDIR /app/src/rs
RUN ls
COPY ./Cargo.toml ./Cargo.lock  ./
RUN cargo check
RUN cargo build

# Copy everything else into the source
COPY . .
RUN rm -rf src/ cargo.toml README.md tests

# RUN cargo build --featurers btc