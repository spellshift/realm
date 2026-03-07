import re

with open('tavern/internal/www/src/pages/shellv2/components/SshTerminal.tsx', 'r') as f:
    content = f.read()

# Replace seqId logic
new_ws_handlers = r"""            ws.onmessage = (event) => {
                // Check if the data is a Blob
                if (event.data instanceof Blob) {
                    const reader = new FileReader();
                    reader.onload = () => {
                        const text = reader.result as string;
                        term.write(text);
                    };
                    reader.readAsText(event.data);
                } else {
                    term.write(event.data);
                }
            };

            ws.onclose = (event) => {
                if (isDisposed) return;

                term.writeln(`\r\n\x1b[31mConnection closed (Code: ${event.code}, Reason: ${event.reason})\x1b[0m`);
                if (event.code !== 1000) {
                    term.writeln(`\r\n\x1b[33mReconnecting in 3 seconds...\x1b[0m`);
                    reconnectTimeout = setTimeout(connect, 3000);
                }
            };

            ws.onerror = (e) => {
                console.error("SSH WebSocket Error", e);
                // term.writeln(`\r\n\x1b[31mWebSocket Error.\x1b[0m`);
                // We let onclose handle the reconnection logic
            };
        };

        connect();

        const handleResize = () => fitAddon.fit();
        window.addEventListener("resize", handleResize);

        term.onData((data) => {
            if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
                seqId++; // Increment seq_id on sending messages
                wsRef.current.send(data);
            }
        });"""

content = content.replace(r"""            ws.onmessage = (event) => {
                seqId++; // Increment seq_id on receiving messages

                // Check if the data is a Blob
                if (event.data instanceof Blob) {
                    const reader = new FileReader();
                    reader.onload = () => {
                        const text = reader.result as string;
                        term.write(text);
                    };
                    reader.readAsText(event.data);
                } else {
                    term.write(event.data);
                }
            };

            ws.onclose = (event) => {
                if (isDisposed) return;

                term.writeln(`\r\n\x1b[31mConnection closed (Code: ${event.code}, Reason: ${event.reason})\x1b[0m`);
                if (event.code !== 1000) {
                    term.writeln(`\r\n\x1b[33mReconnecting in 3 seconds...\x1b[0m`);
                    reconnectTimeout = setTimeout(connect, 3000);
                }
            };

            ws.onerror = (e) => {
                console.error("SSH WebSocket Error", e);
                // term.writeln(`\r\n\x1b[31mWebSocket Error.\x1b[0m`);
                // We let onclose handle the reconnection logic
            };
        };

        connect();

        const handleResize = () => fitAddon.fit();
        window.addEventListener("resize", handleResize);

        term.onData((data) => {
            if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
                seqId++; // Increment seq_id on sending messages
                wsRef.current.send(data);
            }
        });""", new_ws_handlers)

with open('tavern/internal/www/src/pages/shellv2/components/SshTerminal.tsx', 'w') as f:
    f.write(content)
