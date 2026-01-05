# Dependency Cache
FROM golang:1.24.0-bookworm as base
WORKDIR /app
RUN mkdir -p /app/build /app/cdn
COPY ./go.mod /app/go.mod
COPY ./go.sum /app/go.sum
RUN go mod download

# Build Cache
FROM base as build-cache
COPY ./tavern /app/tavern

# Production Build
FROM build-cache as prod-build
RUN go build -ldflags='-w -extldflags "-static"' -o /app/build/tavern ./tavern

# Production
FROM debian:bookworm as production
WORKDIR /app
CMD ["/app/tavern"]
EXPOSE 80 443 8080
RUN apt-get update -y && apt-get install -y ca-certificates
COPY --from=prod-build /app/build/tavern /app/tavern
