# Install pivxd and get it ready to host a masternode

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y \
    wget \
    bash \
    && rm -rf /var/lib/apt/lists/*
RUN wget "https://github.com/PIVX-Project/PIVX/releases/download/v5.6.1/pivx-5.6.1-x86_64-linux-gnu.tar.gz"
RUN tar xf pivx-5.6.1-x86_64-linux-gnu.tar.gz
RUN rm pivx-5.6.1-x86_64-linux-gnu.tar.gz
WORKDIR /app/pivx-5.6.1
RUN ./install-params.sh
# download snapshot (For some reason the .zip version is much faster to download)
RUN wget "https://snapshot.rockdev.org/PIVXsnapshotLatest.zip"
WORKDIR /root/.pivx/
RUN tar xf PIVXsnapshotLatest.tgz
CMD ["/app/pivx-5.6.1/bin/pivxd"]
