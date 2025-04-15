FROM rust:latest as builder

RUN apt-get update && \
    apt-get install -y \
    libgtk-4-dev \
    libadwaita-1-dev \
    pkg-config \
    libdbus-1-dev \
    libegl-dev \
    mesa-utils \
    libgl1-mesa-dev \
    libglvnd-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y \
    libgtk-4-1 \
    libadwaita-1-0 \
    libdbus-1-3 \
    libegl1 \
    mesa-utils \
    libgl1-mesa-dri \
    libglx-mesa0 \
    libglvnd0 \
    x11-apps \
    dbus-x11 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/calculator_rust_project /usr/local/bin/

CMD ["dbus-run-session", "calculator_rust_project"]