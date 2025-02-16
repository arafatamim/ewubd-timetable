FROM lukemathwalker/cargo-chef:latest-rust-bookworm AS chef

WORKDIR /app

FROM chef AS planner
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src
RUN cargo chef prepare

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release
COPY . .
RUN cargo build --release
RUN mv ./target/release/ewubd-timetable-calendar ./app

FROM debian:bookworm-slim AS runtime

RUN apt update -y && apt install openssl -y

COPY --from=builder /app/app /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/app"]
