FROM rust:1.89-bookworm

WORKDIR /workspace

COPY . .

RUN cargo fetch

EXPOSE 3000

CMD ["cargo", "run", "-p", "grand-edge-api"]
