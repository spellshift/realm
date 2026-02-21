import { Terminal } from "@xterm/xterm";
import { AttachAddon } from 'xterm-addon-attach';
import { useState, useEffect, useRef } from 'react';
import { useParams } from "react-router-dom";
import { Steps } from "@chakra-ui/react";
import { toaster } from "../../components/ui/toaster";
import '@xterm/xterm/css/xterm.css';
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import Button from "../../components/tavern-base-ui/button/Button";
import Badge from "../../components/tavern-base-ui/badge/Badge";
import Breadcrumbs from "../../components/Breadcrumbs";

const Shell = () => {
    const { shellId } = useParams();

    const [wsIsOpen, setWsIsOpen] = useState(false);
    const [latency, setLatency] = useState<number | null>(null);
    const ws = useRef<WebSocket | null>(null);
    const pingWs = useRef<WebSocket | null>(null);
    const termRef = useRef<Terminal | null>(null);
    if (termRef.current === null) {
        termRef.current = new Terminal();
    }

    // Setup Shell WebSocket
    useEffect(() => {
        if (!ws.current) {
            const scheme = window.location.protocol === "https:" ? 'wss' : 'ws';
            const socket = new WebSocket(`${scheme}://${window.location.host}/shell/ws?shell_id=${shellId}`);

            socket.onopen = (e) => {
                setWsIsOpen(true);
                toaster.create({
                    title: 'Shell Connected',
                    description: 'Only output after your connection is displayed, so you may need to enter a newline to see the prompt',
                    type: 'success',
                    duration: 6000,
                })
                const attachAddon = new AttachAddon(socket);
                termRef.current?.loadAddon(attachAddon);
            };
            socket.onerror = (e) => {
                toaster.create({
                    title: 'Shell Connection Error',
                    description: `Something went wrong with the underlying connection to the shell (${e.type})`,
                    type: 'error',
                    duration: 6000,
                })
            }
            socket.onclose = (e) => {
                setWsIsOpen(false);
                toaster.create({
                    title: 'Shell Closed',
                    description: `Your shell connection has been closed, however the shell may still be available (${e.type})`,
                    type: 'info',
                    duration: 6000,
                })
            }

            ws.current = socket;
        }

        // Cleanup
        return () => {
            if (ws.current) {
                ws.current.close();
                ws.current = null;
            }
        }
    }, [shellId]);

    // Setup Ping WebSocket and Loop
    useEffect(() => {
        if (!pingWs.current) {
            const scheme = window.location.protocol === "https:" ? 'wss' : 'ws';
            const socket = new WebSocket(`${scheme}://${window.location.host}/shell/ping?shell_id=${shellId}`);
            socket.binaryType = 'arraybuffer';

            socket.onmessage = (ev) => {
                // We expect the payload to be the timestamp we sent
                // It comes back as binary (ArrayBuffer) because the backend writes BinaryMessage
                try {
                    const dec = new TextDecoder("utf-8");
                    let sentAtStr = "";
                    if (ev.data instanceof ArrayBuffer) {
                        sentAtStr = dec.decode(ev.data);
                    } else if (typeof ev.data === "string") {
                        sentAtStr = ev.data;
                    }

                    const sentAt = parseInt(sentAtStr);
                    if (!isNaN(sentAt)) {
                        const now = Date.now();
                        setLatency(now - sentAt);
                    }
                } catch (e) {
                    console.error("Failed to parse ping response", e);
                }
            };

            pingWs.current = socket;
        }

        const timer = setInterval(() => {
            if (pingWs.current && pingWs.current.readyState === WebSocket.OPEN) {
                // Send current timestamp as string/bytes
                const now = Date.now().toString();
                pingWs.current.send(now);
            }
        }, 2000);

        return () => {
            clearInterval(timer);
            if (pingWs.current) {
                pingWs.current.close();
                pingWs.current = null;
            }
        };
    }, [shellId]);

    const renderTerminal = (div: HTMLDivElement) => { if (div) { termRef.current?.open(div); } };

    //TODO: Expand to fetch active users for this page
    return (
        <>
            <Breadcrumbs pages={[{
                label: "Shell",
                link: "/shell"
            }]} />
            <div className="border-b-2 border-gray-200 pb-6 sm:flex flex-row sm:items-center sm:justify-between">
                <div className="flex flex-col gap-2">
                    <div className="flex flex-row gap-4 items-center">
                        <h3 className="text-xl font-semibold leading-6 text-gray-900">Shell for id:{shellId}</h3>
                        <Badge badgeStyle={{ color: "purple" }} >BETA FEATURE</Badge>
                        {latency !== null && (
                            <Badge badgeStyle={{ color: latency < 200 ? "green" : "red" }}>
                                {latency}ms
                            </Badge>
                        )}
                    </div>
                    <p className="max-w-2xl text-sm">Start by clicking inside the terminal, you may need to enter a newline to see the terminal prompt.</p>
                </div>
                <a title="Report a Bug" target="_blank" href="https://github.com/spellshift/realm/issues/new?template=bug_report.md&labels=bug&title=%5Bbug%5D%20Reverse%20Shell%3A%20%3CYOUR%20ISSUE%3E" rel="noreferrer">
                    <Button buttonStyle={{ color: "gray", size: "md" }}>
                        Report a bug
                    </Button>
                </a>
            </div>

            {
                wsIsOpen ?
                    <div id="terminal" className="w-full bg-gray-500 h-96" ref={renderTerminal} /> :
                    <EmptyState label="Connecting..." type={EmptyStateType.loading} />
            }
        </>
    );
}
export default Shell;
