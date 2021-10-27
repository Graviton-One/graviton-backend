FROM rustlang/rust:nightly-buster as builder

RUN USER=root cargo new --bin pollers
WORKDIR /pollers

# copy over your manifests
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src
#COPY ./contracts ./contracts
#COPY . .

RUN cargo build --release --bin gton_price

FROM debian:buster-slim

RUN apt-get update && \
    apt-get --assume-yes install \
        make \
        libpq5 \
        libpq-dev \
        -qqy \
        --no-install-recommends
RUN apt-get update && apt-get -y install ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

COPY --from=builder /pollers/target/release/gton_price /pollers/gton_price
WORKDIR /pollers/
EXPOSE 8088
CMD ["/pollers/gton_price"]
