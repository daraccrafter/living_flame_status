FROM rust:latest
WORKDIR /app
COPY . .
RUN cargo build --release

CMD ["./target/release/living_flame_status"]

