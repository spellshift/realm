# Dependency Cache
FROM golang:1.26.2-trixie as base
WORKDIR /app
RUN mkdir -p /app/build /app/cdn
COPY ./go.mod /app/go.mod
COPY ./go.sum /app/go.sum
RUN go mod download

# Build Cache
FROM base as build-cache
COPY ./tavern /app/tavern
COPY ./bin /app/bin

# Production Build
FROM build-cache as prod-build
RUN CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -ldflags='-w -s -extldflags "-static"' -o /app/tavern/tools/linux/socks5 ./bin/socks5
RUN CGO_ENABLED=0 GOOS=darwin GOARCH=amd64 go build -ldflags='-w -s -extldflags "-static"' -o /app/tavern/tools/macos/socks5 ./bin/socks5
RUN CGO_ENABLED=0 GOOS=freebsd GOARCH=amd64 go build -ldflags='-w -s -extldflags "-static"' -o /app/tavern/tools/bsd/socks5 ./bin/socks5
RUN CGO_ENABLED=0 GOOS=windows GOARCH=amd64 go build -ldflags='-w -s -extldflags "-static"' -o /app/tavern/tools/windows/socks5.exe ./bin/socks5
RUN go build -ldflags='-w -extldflags "-static"' -o /app/build/tavern ./tavern

# Production
FROM debian:trixie as production
WORKDIR /app
CMD ["/app/tavern"]
RUN apt-get update -y && apt-get install -y ca-certificates
COPY --from=prod-build /app/build/tavern /app/tavern
