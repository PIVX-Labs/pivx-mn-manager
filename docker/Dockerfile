ARG MN_IP
ARG MN_PRIV_KEY

FROM debian:bookworm-slim

ARG MN_IP
ARG MN_PRIV_KEY

RUN test -n "$MN_IP" || { echo "MN_IP is required"; exit 1; }
RUN test -n "$MN_PRIV_KEY" || { echo "MN_PRIV_KEY is required"; exit 1; }

WORKDIR /app

RUN apt-get update && apt-get install -y \
    wget \
    bash \
    unzip \
    && rm -rf /var/lib/apt/lists/*

RUN wget "https://github.com/PIVX-Project/PIVX/releases/download/v5.6.1/pivx-5.6.1-x86_64-linux-gnu.tar.gz"
COPY SHA256SUM.asc .
RUN grep " pivx-5.6.1-x86_64-linux-gnu.tar.gz\$" SHA256SUM.asc | sha256sum -c -
RUN tar xf pivx-5.6.1-x86_64-linux-gnu.tar.gz
RUN rm pivx-5.6.1-x86_64-linux-gnu.tar.gz SHA256SUM.asc
WORKDIR /app/pivx-5.6.1
RUN cp bin/* /bin/
RUN ./install-params.sh
# download snapshot (For some reason the .zip version is much faster to download)
RUN wget "https://snapshot.rockdev.org/PIVXsnapshotLatest.zip"
WORKDIR /root/.pivx/
RUN unzip /app/pivx-5.6.1/PIVXsnapshotLatest.zip
COPY pivx.template.conf .
RUN sed -i 's+$IP+'"$MN_IP"'+g' pivx.template.conf
RUN sed -i 's+$PRIV_KEY+'"$MN_PRIV_KEY"'+g' pivx.template.conf
CMD ["pivxd"]