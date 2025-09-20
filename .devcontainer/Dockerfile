FROM mcr.microsoft.com/devcontainers/base:trixie


COPY library-scripts/*.sh library-scripts/*.env /tmp/library-scripts/

# Install Jekyll
RUN bash /tmp/library-scripts/jekyll-debian.sh

# Persist Bash History
RUN SNIPPET="export PROMPT_COMMAND='history -a' && export HISTFILE=/commandhistory/.bash_history" \
    && echo $SNIPPET >> "/root/.bashrc"

# Install additional dependencies
#   - Git: git-lfs
#   - Rust Cross-compilation: musl-tools gcc-mingw-w64
#   - Protobuf: protobuf-compiler protoc-gen-go protoc-gen
RUN apt update && apt install -y git-lfs musl-tools gcc-mingw-w64 protobuf-compiler protoc-gen-go protoc-gen-go-grpc

RUN echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] https://packages.cloud.google.com/apt cloud-sdk main" \
    | tee -a /etc/apt/sources.list.d/google-cloud-sdk.list && curl https://packages.cloud.google.com/apt/doc/apt-key.gpg | \
    gpg --dearmor -o /usr/share/keyrings/cloud.google.gpg && apt-get update -y && apt-get install google-cloud-cli -y

RUN echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/hashicorp-archive-keyring.gpg] https://apt.releases.hashicorp.com $(grep -oP '(?<=UBUNTU_CODENAME=).*' /etc/os-release || lsb_release -cs) main" \
        | tee /etc/apt/sources.list.d/hashicorp.list && apt-get update && apt-get install -y \
        gnupg software-properties-common && wget -O- https://apt.releases.hashicorp.com/gpg | \
        gpg --dearmor | tee /usr/share/keyrings/hashicorp-archive-keyring.gpg > /dev/null && \
        apt update && apt-get install -y terraform
