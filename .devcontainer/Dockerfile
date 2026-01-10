FROM mcr.microsoft.com/devcontainers/base:trixie

ARG ZIG_VERSION=0.16.0-dev.1859+212968c57

ENV GOPATH=/go
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=$GOPATH/bin:/usr/local/bin:/usr/local/go/bin:/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.91.1 \
    GOLANG_VERSION=1.24.3

COPY library-scripts/*.sh library-scripts/*.env /tmp/library-scripts/

# Install Jekyll
RUN bash /tmp/library-scripts/jekyll-debian.sh

# Install additional dependencies
#   - Git: git-lfs
#   - Rust Cross-compilation: musl-tools gcc-mingw-w64
#   - Protobuf: protobuf-compiler protoc-gen-go protoc-gen
RUN apt update && apt install -y git-lfs musl-tools gcc-mingw-w64 protobuf-compiler protoc-gen-go protoc-gen-go-grpc

RUN echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] https://packages.cloud.google.com/apt cloud-sdk main" \
    | tee -a /etc/apt/sources.list.d/google-cloud-sdk.list && curl https://packages.cloud.google.com/apt/doc/apt-key.gpg | \
    gpg --dearmor -o /usr/share/keyrings/cloud.google.gpg && apt-get update -y && apt-get install google-cloud-cli -y

RUN apt-get update && apt-get install -y gnupg && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/hashicorp-archive-keyring.gpg] https://apt.releases.hashicorp.com $(grep -oP '(?<=UBUNTU_CODENAME=).*' /etc/os-release || lsb_release -cs) main" | \
        tee /etc/apt/sources.list.d/hashicorp.list && wget -O- https://apt.releases.hashicorp.com/gpg | \
        gpg --dearmor | tee /usr/share/keyrings/hashicorp-archive-keyring.gpg > /dev/null && \
        apt update && apt-get install -y terraform

RUN wget "https://ziglang.org/builds/zig-$(uname -m)-linux-$ZIG_VERSION.tar.xz" -O /tmp/zig.tar.xz && \
    mkdir /usr/local/bin/zig && tar -xvf "/tmp/zig.tar.xz" --strip-components=1 -C /usr/local/bin/zig && \
    echo "export PATH=\"\$PATH:/usr/local/bin/zig/\"" >> /etc/bash.bashrc

# Install Golang
# https://github.com/docker-library/golang/blob/master/1.24/trixie/Dockerfile
RUN set -eux; \
	now="$(date '+%s')"; \
	arch="$(dpkg --print-architecture)"; arch="${arch##*-}"; \
	url=; \
	case "$arch" in \
		'amd64') \
			url="https://dl.google.com/go/go$GOLANG_VERSION.linux-amd64.tar.gz"; \
			sha256='3333f6ea53afa971e9078895eaa4ac7204a8c6b5c68c10e6bc9a33e8e391bdd8'; \
			;; \
		'arm64') \
			url="https://dl.google.com/go/go$GOLANG_VERSION.linux-arm64.tar.gz"; \
			sha256='a463cb59382bd7ae7d8f4c68846e73c4d589f223c589ac76871b66811ded7836'; \
			;; \
		*) echo >&2 "error: unsupported architecture '$arch' (likely packaging update needed)"; exit 1 ;; \
	esac; \
	wget -O go.tgz.asc "$url.asc"; \
	wget -O go.tgz "$url" --progress=dot:giga; \
	echo "$sha256 *go.tgz" | sha256sum -c -; \
	\
# https://github.com/golang/go/issues/14739#issuecomment-324767697
	GNUPGHOME="$(mktemp -d)"; export GNUPGHOME; \
# https://www.google.com/linuxrepositories/
	gpg --batch --keyserver keyserver.ubuntu.com --recv-keys 'EB4C 1BFD 4F04 2F6D DDCC  EC91 7721 F63B D38B 4796'; \
# let's also fetch the specific subkey of that key explicitly that we expect "go.tgz.asc" to be signed by, just to make sure we definitely have it
	gpg --batch --keyserver keyserver.ubuntu.com --recv-keys '2F52 8D36 D67B 69ED F998  D857 78BD 6547 3CB3 BD13'; \
	gpg --batch --verify go.tgz.asc go.tgz; \
	gpgconf --kill all; \
	rm -rf "$GNUPGHOME" go.tgz.asc; \
	\
	tar -C /usr/local -xzf go.tgz; \
	rm go.tgz; \
	mkdir -p "$GOPATH/src" "$GOPATH/bin" && chmod -R 1777 "$GOPATH" && \
	go version;

# Install rust
# https://github.com/rust-lang/docker-rust/blob/master/stable/trixie/Dockerfile
RUN set -eux; \
    \
    arch="$(dpkg --print-architecture)"; \
    case "$arch" in \
        'amd64') \
            rustArch='x86_64-unknown-linux-gnu'; \
            rustupSha256='20a06e644b0d9bd2fbdbfd52d42540bdde820ea7df86e92e533c073da0cdd43c'; \
            ;; \
        'arm64') \
            rustArch='aarch64-unknown-linux-gnu'; \
            rustupSha256='e3853c5a252fca15252d07cb23a1bdd9377a8c6f3efa01531109281ae47f841c'; \
            ;; \
        *) \
            echo >&2 "unsupported architecture: $arch"; \
            exit 1; \
            ;; \
    esac; \
    \
    url="https://static.rust-lang.org/rustup/archive/1.28.2/${rustArch}/rustup-init"; \
    wget --progress=dot:giga "$url"; \
    echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION --default-host ${rustArch}; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version; \
	rustup target add aarch64-unknown-linux-gnu \
		x86_64-unknown-linux-musl x86_64-pc-windows-gnu \
		x86_64-pc-windows-msvc aarch64-apple-darwin x86_64-apple-darwin; \
    rustup component add rustfmt clippy; \
	cargo install cargo-zigbuild; \
	cargo +1.89.0 install xwin --locked

# Install node
RUN bash /tmp/library-scripts/node-debian.sh

# Install Python
# https://github.com/docker-library/python/blob/master/3.12/slim-trixie/Dockerfile
RUN apt update && apt install -y python3

RUN rm -rf /var/lib/apt/lists/* && rm -rf /tmp/*
