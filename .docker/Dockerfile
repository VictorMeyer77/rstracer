FROM rust:1.82.0-bullseye

# Basic dependencies

RUN apt-get update && apt-get install -y \
    git \
    build-essential \
    python3-pip \
    sudo \
    systemd \
    systemd-sysv

# Rstracer dependencies

RUN apt-get install -y \
    libpcap-dev \
    lsof \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# CICD dependencies

RUN pip install --no-cache-dir pre-commit
RUN rustup component add rustfmt
RUN rustup component add clippy
RUN cargo install grcov
RUN mkdir .coverage

# Install project files

ADD ../.docker/ /rstracer/.docker
ADD ../.github /rstracer/.github
ADD ../etc /rstracer/etc
ADD ../logs /rstracer/logs
ADD ../lsof /rstracer/lsof
ADD ../netstat /rstracer/netstat
ADD ../network /rstracer/network
ADD ../ps /rstracer/ps
ADD ../rstracer /rstracer/rstracer
ADD ../.gitignore /rstracer/.gitignore
ADD ../.pre-commit-config.yaml /rstracer/.pre-commit-config.yaml
ADD ../Cargo.toml /rstracer/Cargo.toml
ADD ../CHANGELOG.md /rstracer/CHANGELOG.md
ADD ../LICENSE /rstracer/LICENSE
ADD ../README.md /rstracer/README.md
ADD ../rstracer.toml /rstracer/rstracer.toml

WORKDIR /rstracer
VOLUME ["/rstracer/.coverage"]

# Launch

ENTRYPOINT ["/sbin/init"]
