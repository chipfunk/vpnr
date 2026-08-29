FROM rust:trixie AS BUILD

WORKDIR /app

COPY . /app

RUN cargo build --release


FROM debian:trixie AS RUN

COPY --from=BUILD /app/target/release/vpnr /usr/local/bin/vpnr

# RUN strip -s /usr/local/bin/vpnr

ENTRYPOINT [ "/usr/local/bin/vpnr" ]
CMD [ "help" ]
