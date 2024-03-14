import { PageWrapper } from "../../components/page-wrapper";
import { PageNavItem } from "../../utils/enums";
import { Terminal } from "@xterm/xterm";
import { AttachAddon } from 'xterm-addon-attach';
import { useState, useEffect, useRef } from 'react';
import { useParams } from "react-router-dom";
import '@xterm/xterm/css/xterm.css'
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";

const Shell = () => {
    const { shellId } = useParams();

    const [wsIsOpen, setWsIsOpen] = useState(false);
    const ws = useRef<WebSocket | null>(null);
    const termRef = useRef<Terminal | null>(null);
    if (termRef.current === null) {
        termRef.current = new Terminal();
    }

    // Setup WebSocket
    useEffect(() => {
        if (!ws.current) {
            const scheme = window.location.protocol === "https:" ? 'wss' : 'ws';
            const socket = new WebSocket(`${scheme}://${window.location.hostname}/shell/ws?shell_id=${shellId}`);

            socket.onopen = (e) => {
                setWsIsOpen(true);
                const attachAddon = new AttachAddon(socket);
                termRef.current?.loadAddon(attachAddon);
            };
            ws.current = socket;

            socket.onclose = (e) => {
                setWsIsOpen(false);
            }
        }
    }, [shellId]);

    const renderTerminal = (div: HTMLDivElement) => { if (div) { termRef.current?.open(div); } };

    return (
        <PageWrapper currNavItem={PageNavItem.dashboard}>
            <div className="border-b border-gray-200 pb-6 sm:flex sm:items-center sm:justify-between">
                <h3 className="text-xl font-semibold leading-6 text-gray-900">Shell <i>[BETA]</i></h3>
            </div>
            {!wsIsOpen ? <EmptyState label="Connecting..." type={EmptyStateType.loading} /> : null}
            <div id="terminal" className="w-full bg-gray-500 h-96" ref={renderTerminal} />
        </PageWrapper>
    );
}
export default Shell;
