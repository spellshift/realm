FROM debian:buster as staging
WORKDIR /app
RUN sh -c 'curl -s https://packagecloud.io/install/repositories/github/git-lfs/script.deb.sh | sh'
RUN apt update && apt install -y git gis-lfs
COPY . .
RUN git lfs pull

# Dependency Cache
FROM golang:1.20.2-buster as base
WORKDIR /app
RUN mkdir -p /app/build /app/cdn
COPY --from=staging /app/go.mod /app/go.mod
COPY --from=staging /app/go.sum /app/go.sum
RUN go mod download

# Build Cache
FROM base as build-cache
COPY --from=staging /app/tavern /app/tavern

# Production Build
FROM build-cache as prod-build
RUN go build -ldflags='-w -extldflags "-static"' -o /app/build/tavern ./tavern

# Production
FROM debian:buster as production
WORKDIR /app
CMD ["/app/tavern"]
RUN apt-get update -y && apt-get install -y ca-certificates
COPY --from=prod-build /app/build/tavern /app/tavern

