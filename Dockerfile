FROM rust:trixie AS BUILD

WORKDIR /app

COPY Cargo.toml /app
COPY Cargo.lock /app
COPY README.md /app
COPY /src /app/src/

RUN cargo build --release
RUN strip -s target/release/vpnr


FROM debian:trixie AS RUN

COPY --from=BUILD /app/target/release/vpnr /usr/local/bin/vpnr

ENTRYPOINT [ "/usr/local/bin/vpnr" ]
CMD [ "help" ]
