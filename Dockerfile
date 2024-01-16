FROM rust:1.75.0-buster as builder

WORKDIR /usr/src/listmonk-mailersend

COPY . .

RUN cargo install --path .

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/listmonk-mailersend /usr/local/bin/listmonk-mailersend

ENV PORT=9000
ENV HOST=0.0.0.0

EXPOSE 9000
ENTRYPOINT ["listmonk-mailersend"]
