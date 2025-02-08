FROM mcr.microsoft.com/devcontainers/base:bookworm@sha256:6155a486f236fd5127b76af33086029d64f64cf49dd504accb6e5f949098eb7e


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
