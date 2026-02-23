import "@xterm/xterm/css/xterm.css";
import React from "react";

interface ShellTerminalProps {
    termRef: React.RefObject<HTMLDivElement>;
    completions: string[];
    showCompletions: boolean;
    completionPos: { x: number; y: number };
    completionIndex: number;
    completionsListRef: React.RefObject<HTMLUListElement>;
}

const ShellTerminal = ({
    termRef,
    completions,
    showCompletions,
    completionPos,
    completionIndex,
    completionsListRef
}: ShellTerminalProps) => {
    return (
        <div className="flex-grow rounded overflow-hidden relative border border-[#333]">
            <div ref={termRef} style={{ height: "100%", width: "100%" }} />

            {showCompletions && (
                <div style={{
                    position: "absolute",
                    top: completionPos.y,
                    left: completionPos.x,
                    background: "#252526",
                    border: "1px solid #454545",
                    zIndex: 1000,
                    maxHeight: "200px",
                    overflowY: "auto",
                    boxShadow: "0 4px 6px rgba(0,0,0,0.3)",
                    color: "#cccccc",
                    fontFamily: 'Menlo, Monaco, "Courier New", monospace',
                    fontSize: "14px"
                }}>
                    <ul ref={completionsListRef} style={{ listStyle: "none", margin: 0, padding: 0 }}>
                        {completions.map((c, i) => (
                            <li key={i} style={{
                                padding: "4px 8px",
                                background: i === completionIndex ? "#094771" : "transparent",
                                cursor: "pointer"
                            }}>
                                {c}
                            </li>
                        ))}
                    </ul>
                </div>
            )}
        </div>
    );
};

export default ShellTerminal;
