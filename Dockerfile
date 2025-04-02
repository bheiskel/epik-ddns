FROM rust:alpine AS builder
WORKDIR /app
COPY . /app
RUN apk add --no-cache openssl openssl-dev openssl-libs-static musl-dev 
RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=builder /app/target/release/epik-ddns /
ENTRYPOINT ["./epik-ddns"]

