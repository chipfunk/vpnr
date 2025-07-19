FROM rust:bookworm AS BUILD

WORKDIR /app

COPY Cargo.toml /app
COPY Cargo.lock /app
COPY README.md /app
COPY /src /app/src/

RUN cargo build --release
RUN strip -s target/release/vpnr


FROM debian:bookworm AS RUN

COPY --from=BUILD /app/target/release/vpnr /usr/local/bin/vpnr

RUN mkdir /vpnr
WORKDIR /vpnr

ENTRYPOINT [ "/usr/local/bin/vpnr" ]
