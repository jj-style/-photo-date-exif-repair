FROM rust:1.66 as builder
WORKDIR /app
COPY . .
RUN cargo install --path .

FROM debian:bullseye
RUN apt-get update && apt-get install -y exiftool && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/photo-date-exif-repair /usr/local/bin/photo-date-exif-repair
CMD ["photo-date-exif-repair"]