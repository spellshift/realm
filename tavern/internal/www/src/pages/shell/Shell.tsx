import { PageWrapper } from "../../components/page-wrapper";
import { Terminal } from "@xterm/xterm";
import { AttachAddon } from 'xterm-addon-attach';
import { useState, useEffect, useRef } from 'react';
import { useParams } from "react-router-dom";
import { useToast } from "@chakra-ui/react";
import '@xterm/xterm/css/xterm.css';
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import Button from "../../components/tavern-base-ui/button/Button";
import Badge from "../../components/tavern-base-ui/badge/Badge";
import Breadcrumbs from "../../components/Breadcrumbs";

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
            const socket = new WebSocket(`${scheme}://${window.location.host}/shell/ws?shell_id=${shellId}`);

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

    //TODO: Expand to fetch active users for this page
    return (
        <PageWrapper>
            <Breadcrumbs pages={[{
                label: "Shell",
                link: "/shell"
            }]} />
            <div className="border-b-2 border-gray-200 pb-6 sm:flex flex-row sm:items-center sm:justify-between">
                <div className="flex flex-col gap-2">
                    <div className="flex flex-row gap-4 items-center">
                        <h3 className="text-xl font-semibold leading-6 text-gray-900">Shell for id:{shellId}</h3>
                        <Badge badgeStyle={{ color: "purple" }} >BETA FEATURE</Badge>
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
        </PageWrapper >
    );
}
export default Shell;
