import { Terminal } from "@xterm/xterm";
import { AttachAddon } from 'xterm-addon-attach';
import { useState, useEffect, useRef } from 'react';
import { useSearchParams } from "react-router-dom";
import { useToast } from "@chakra-ui/react";
import '@xterm/xterm/css/xterm.css';
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import Badge from "../../components/tavern-base-ui/badge/Badge";

const SshTerminal = () => {
    const [searchParams] = useSearchParams();
    const portalId = searchParams.get('portal_id');
    const target = searchParams.get('target');
    const toast = useToast();

    const [wsIsOpen, setWsIsOpen] = useState(false);
    const ws = useRef<WebSocket | null>(null);
    const termRef = useRef<Terminal | null>(null);

    if (termRef.current === null) {
        termRef.current = new Terminal();
    }

    // Setup SSH WebSocket
    useEffect(() => {
        if (!portalId || !target) {
            toast({
                title: 'Missing parameters',
                description: 'Both portal_id and target are required',
                status: 'error',
                duration: null,
                isClosable: true,
            });
            return;
        }

        if (!ws.current) {
            const scheme = window.location.protocol === "https:" ? 'wss' : 'ws';
            const socket = new WebSocket(`${scheme}://${window.location.host}/portals/ssh/ws?portal_id=${portalId}&target=${encodeURIComponent(target)}`);

            socket.onopen = (e) => {
                setWsIsOpen(true);
                toast({
                    title: 'SSH Connected',
                    description: `Connected to ${target}`,
                    status: 'success',
                    duration: 6000,
                    isClosable: true,
                });
                const attachAddon = new AttachAddon(socket);
                termRef.current?.loadAddon(attachAddon);
            };
            socket.onerror = (e) => {
                toast({
                    title: 'SSH Connection Error',
                    description: `Failed to connect to ${target}`,
                    status: 'error',
                    duration: 6000,
                    isClosable: true,
                })
            }
            socket.onclose = (e) => {
                setWsIsOpen(false);
                toast({
                    title: 'SSH Closed',
                    description: `Connection to ${target} closed`,
                    status: 'info',
                    duration: 6000,
                    isClosable: true,
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
    }, [portalId, target, toast]);

    const renderTerminal = (div: HTMLDivElement) => { if (div) { termRef.current?.open(div); } };

    if (!portalId || !target) {
         return <EmptyState label="Missing parameters" type={EmptyStateType.error} />;
    }

    return (
        <div className="flex flex-col h-screen p-5 bg-[#1e1e1e] text-[#d4d4d4]">
            <div className="border-b-2 border-gray-600 pb-4 mb-4 sm:flex flex-row sm:items-center sm:justify-between">
                <div className="flex flex-col gap-2">
                    <div className="flex flex-row gap-4 items-center">
                        <h3 className="text-xl font-semibold leading-6 text-white">SSH Session</h3>
                        <Badge badgeStyle={{ color: "purple" }} >{target}</Badge>
                        <Badge badgeStyle={{ color: "gray" }} >Portal {portalId}</Badge>
                    </div>
                </div>
            </div>

            {
                wsIsOpen ?
                    <div id="terminal" className="w-full h-full flex-grow" ref={renderTerminal} /> :
                    <div className="flex items-center justify-center h-full">
                        <div className="text-xl text-gray-400">Connecting...</div>
                    </div>
            }
        </div>
    );
}

export default SshTerminal;