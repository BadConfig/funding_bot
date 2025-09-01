FROM alpine:latest AS builder
RUN apk add --no-cache ca-certificates

FROM scratch
ARG BIN
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY target/x86_64-unknown-linux-musl/release/${BIN} /service
CMD ["/service"]
