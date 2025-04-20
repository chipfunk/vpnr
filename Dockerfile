FROM rust:bookworm AS BUILD

WORKDIR /app

COPY Cargo.toml /app
COPY Cargo.lock /app
COPY /src /app/src/

RUN ls -la

RUN cargo build --release
RUN strip -s target/release/vpnr


FROM scratch

COPY --from=BUILD /app/target/release/vpnr /vpnr
