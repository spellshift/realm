import { Terminal } from "@xterm/xterm";
import { MutableRefObject } from "react";
import { WebsocketControlFlowSignal, WebsocketMessage, WebsocketMessageKind } from "../../websocket";

interface HandleAdapterMessageParams {
    msg: WebsocketMessage;
    term: Terminal | null;
    lastBufferHeight: MutableRefObject<number>;
    redrawLine: () => void;
    setPortalId: (id: number | null) => void;
}

export const handleAdapterMessage = ({
    msg,
    term,
    lastBufferHeight,
    redrawLine,
    setPortalId
}: HandleAdapterMessageParams) => {
    if (!term) return;

    // Clear current input line(s) before printing output
    const prevRows = lastBufferHeight.current;
    if (prevRows > 0) {
        term.write(`\x1b[${prevRows}A`);
    }
    term.write("\r\x1b[J");

    // Process message content
    let content = "";
    let color = "";

    switch (msg.kind) {
        case WebsocketMessageKind.Output:
            content = msg.output;
            break;
        case WebsocketMessageKind.TaskError:
            content = msg.error;
            color = "\x1b[38;2;255;0;0m"; // Red
            break;
        case WebsocketMessageKind.Error:
            content = msg.error;
            color = "\x1b[38;2;255;0;0m"; // Red
            break;
        case WebsocketMessageKind.ControlFlow:
            if (msg.signal === WebsocketControlFlowSignal.TaskQueued && msg.message) {
                content = msg.message + "\n";
                color = "\x1b[38;5;178m"; // Purple
            } else if (msg.signal === WebsocketControlFlowSignal.PortalUpgrade && msg.portal_id) {
                setPortalId(msg.portal_id);
            }
            // Handle other control signals if needed
            break;
        case WebsocketMessageKind.OutputFromOtherStream:
            content = msg.output;
            break;
    }

    if (content) {
        const formatted = content.replace(/\n/g, "\r\n");
        if (color) {
            term.write(color + formatted + "\x1b[0m");
        } else {
            term.write(formatted);
        }

        // Ensure there is a newline after output if not present, so prompt is on new line
        if (!content.endsWith('\n')) {
            term.write("\r\n");
        }
    }

    // Reset input line state and redraw it at the bottom
    lastBufferHeight.current = 0;
    redrawLine();
};
