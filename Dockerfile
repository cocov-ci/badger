ARG ARCH=
FROM ${ARCH}debian:buster-slim

ARG BUILDARCH=

COPY ./target/${BUILDARCH}/release/badger /bin/badger

CMD "badger"
