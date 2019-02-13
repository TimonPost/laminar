FROM rust:1.32 as build

# create a new empty shell project
RUN USER=root cargo new --bin laminar
WORKDIR /laminar

# copy over manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src/bin ./src/bin
COPY ./benches ./benches

# caching build deps
RUN cargo build --release
RUN rm src/*.rs

# copy source
COPY ./src ./src
COPY ./examples ./examples

# build for release
RUN cargo clean
RUN cargo build --features="tester" --release

# final base
FROM debian:stretch-slim

# copy the build artifact from the build stage and run
COPY --from=build /laminar/target/release/laminar-tester /usr/bin/laminar-tester
ENTRYPOINT ["laminar-tester"]
