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
RUN go build -ldflags='-w -s -extldflags "-static"' -o /app/build/oauthproxy ./tavern/oauthproxy

# Production
FROM debian:trixie as production
WORKDIR /app
CMD ["/app/oauthproxy"]
RUN apt-get update -y && apt-get install -y ca-certificates
COPY --from=prod-build /app/build/oauthproxy /app/oauthproxy
