# Dependency Cache
FROM golang:1.26.2-trixie as base
WORKDIR /app
RUN mkdir -p /app/build
COPY ./go.mod /app/go.mod
COPY ./go.sum /app/go.sum
RUN go mod download

# Build Cache
FROM base as build-cache
COPY ./tavern /app/tavern

# Production Build
FROM build-cache as prod-build
RUN CGO_ENABLED=0 go build -ldflags='-w -s -extldflags "-static"' -o /app/build/oauthproxy ./tavern/oauthproxy

# Production
FROM debian:trixie as production
WORKDIR /app
RUN apt-get update -y && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=prod-build /app/build/oauthproxy /app/oauthproxy
USER nobody
CMD ["/app/oauthproxy"]
