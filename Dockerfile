FROM debian:buster-slim

COPY ./target/release/badger /bin/badger

ENTRYPOINT ["/bin/badger"]
