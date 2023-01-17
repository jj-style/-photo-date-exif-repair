FROM rust:1.66 as builder
WORKDIR /app
COPY . .
RUN cargo install --path .

FROM debian:bullseye
COPY --from=builder /usr/local/cargo/bin/photo-date-exif-repair /usr/local/bin/photo-date-exif-repair
CMD ["photo-date-exif-repair"]