# Connection Close and TIME-WAIT Solutions

This document outlines five solutions to ensure the server sends a connection close and the client does not hold connections open in the netstat table, specifically for the `http1` redirector and `http1` transport.

## 1. Server-Side Header Instruction (`http1` redirector)
**Approach**: Explicitly add `w.Header().Set("Connection", "close")` before writing the response body in the Go redirector handler.
**Why it works**: The Go standard library's `net/http` server respects this header. After the handler finishes writing the response, the Go HTTP server will immediately dispatch a TCP `FIN` packet to tear down the socket. Because the server is the "active closer", it enters the `TIME_WAIT` state on the OS, and the client socket transitions completely to `CLOSED` once it reads the EOF and cleans up.

## 2. Client-Side Header Instruction (`http1` transport)
**Approach**: Append the `"Connection": "close"` HTTP header to every outgoing request constructed by the Rust transport.
**Why it works**: When the Go `net/http` server receives a request advertising `Connection: close`, it recognizes that the client does not want connection persistence. The server will automatically reply with a `Connection: close` header and proactively close its end of the TCP connection upon completing the response. The server takes the `TIME_WAIT` state, and the client cleans up immediately without lingering sockets.

## 3. Disable Server Keep-Alives (`http1` redirector)
**Approach**: In the Go redirector, locate the `http.Server` initialization (often near `srv.ListenAndServe()`) and explicitly call `srv.SetKeepAlivesEnabled(false)`.
**Why it works**: This globally configures the Go server to operate in short-lived connection mode for this specific redirector binding. The server will forcefully append `Connection: close` to all responses and terminate the TCP sockets after every request. The server guarantees it is the active closer and absorbs all `TIME_WAIT` states.

## 4. Disable Client Connection Pooling (`http1` transport)
**Approach**: Configure the Rust HTTP client builder (e.g., `reqwest::Client`) to explicitly disable its internal connection pool by setting `.pool_max_idle_per_host(0)` and/or `.pool_idle_timeout(None)`.
**Why it works**: Even if the server closes the connection, the client HTTP library might try to hold onto the socket handle for reuse or delay its cleanup (leading to lingering `CLOSE_WAIT` or `ESTABLISHED` sockets if not handled cleanly). Disabling the client pool forces the Rust HTTP library to aggressively drop and destroy the socket struct in memory as soon as the single HTTP exchange concludes, wiping it from the client's netstat immediately.

## 5. TCP Linger `SO_LINGER = 0` via Custom Dial / Hijack
**Approach**:
- **Server side**: Use `http.Hijacker` in the Go redirector to hijack the `net.Conn`, write the response, and forcefully close the raw TCP connection with `SO_LINGER = 0`.
- **Client side**: Implement a custom TCP connector/dialer for the Rust HTTP client that sets the `SO_LINGER = 0` socket option.
**Why it works**: Setting `SO_LINGER` to `0` radically alters the TCP close behavior. Instead of engaging in the standard 4-way `FIN` handshake (which leads to `TIME_WAIT`), dropping the connection immediately unleashes a TCP `RST` (Reset) packet. This instantly incinerates the connection on *both* sides, skipping `TIME_WAIT` entirely and ensuring the sockets immediately vanish from the netstat tables of both the server and the client.
