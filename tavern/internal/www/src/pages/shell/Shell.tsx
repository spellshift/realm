import { PageWrapper } from "../../components/page-wrapper";
import { PageNavItem } from "../../utils/enums";
import { Terminal } from "@xterm/xterm";
import { AttachAddon } from 'xterm-addon-attach';
import { useState, useEffect, useRef } from 'react';
import { useParams } from "react-router-dom";
import { useToast } from "@chakra-ui/react";
import '@xterm/xterm/css/xterm.css';
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import {
    BugAntIcon,
} from '@heroicons/react/24/outline';

const Shell = () => {
    const { shellId } = useParams();
    const toast = useToast();

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
                toast({
                    title: 'Shell Connected',
                    description: 'Only output after your connection is displayed, so you may need to enter a newline to see the prompt',
                    status: 'success',
                    duration: 6000,
                    isClosable: true,
                })
                const attachAddon = new AttachAddon(socket);
                termRef.current?.loadAddon(attachAddon);
            };
            socket.onerror = (e) => {
                toast({
                    title: 'Shell Connection Error',
                    description: `Something went wrong with the underlying connection to the shell (${e.type})`,
                    status: 'error',
                    duration: 6000,
                    isClosable: true,
                })
            }
            socket.onclose = (e) => {
                toast({
                    title: 'Shell Closed',
                    description: `Your shell connection has been closed, however the shell may still be available (${e.type})`,
                    status: 'info',
                    duration: 6000,
                    isClosable: true,
                })
            }
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
                <a title="Report a Bug" target="_blank" href="https://github.com/spellshift/realm/issues/new?template=bug_report.md&labels=bug&title=%5Bbug%5D%20Reverse%20Shell%3A%20%3CYOUR%20ISSUE%3E">
                    <button className="btn-primary" type="button">
                        <BugAntIcon className="text-white w-4 h-4" />
                    </button>
                </a>
            </div>
            {
                wsIsOpen ?
                    <div id="terminal" className="w-full bg-gray-500 h-96" ref={renderTerminal} /> :
                    <EmptyState label="Connecting..." type={EmptyStateType.loading} />
            }
        </PageWrapper >
    );
}
export default Shell;
