import { PageWrapper } from "../../components/page-wrapper";
import { PageNavItem } from "../../utils/enums";
import { Terminal } from "@xterm/xterm";
import React, { useState, useEffect, useRef } from 'react';

const Shell = () => {
    const term = new Terminal();
    const terminalRef = useRef(null);



    // const socket = new WebSocket('wss://' + window.location.href + '/shell/ws');

    useEffect(() => {
        if (term && terminalRef.current) {
            term.open(terminalRef.current);
        }
        const socket = new WebSocket('ws://127.0.0.1:80/shell/ws');

    }, [term, terminalRef]);

    return (
        <PageWrapper currNavItem={PageNavItem.dashboard}>
            <div className="border-b border-gray-200 pb-6 sm:flex sm:items-center sm:justify-between">
                <h3 className="text-xl font-semibold leading-6 text-gray-900">Shell</h3>
            </div>
            <div id="terminal" className="w-full bg-gray-500 h-48" ref={terminalRef}></div>
        </PageWrapper>
    );
}
export default Shell;