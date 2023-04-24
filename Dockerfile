FROM rust AS build

RUN mkdir /work
WORKDIR /work

COPY Cargo.* ./
COPY resources resources
COPY src src

RUN cargo build --release

FROM debian:buster-slim

COPY --from=build /work/target/release/badger /bin/badger

ENTRYPOINT ["/bin/badger"]
