FROM rust:trixie AS BUILD

WORKDIR /app

COPY Cargo.toml /app/
COPY Cargo.lock /app/
COPY src/ /app/src/
COPY README.md /app/

RUN cargo build --release

FROM debian:trixie AS RUN

COPY --from=BUILD /app/target/release/vpnr /usr/local/bin/vpnr

# RUN strip -s /usr/local/bin/vpnr

ENTRYPOINT [ "/usr/local/bin/vpnr" ]
CMD [ "help" ]
