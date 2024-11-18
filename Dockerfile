FROM rust:1.82-bookworm AS builder

WORKDIR /app
COPY . .

RUN cargo build --release

FROM debian:bookworm

RUN apt-get update -y && apt-get install -y \
    ca-certificates \
    jq \
    curl \
    yq \
    && apt-get clean

RUN curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/$(uname -m)/kubectl"
RUN chmod +x ./kubectl
RUN mv ./kubectl /bin

COPY --from=builder /app/target/release/bash-operator /bin

CMD ["sleep", "inf"]
