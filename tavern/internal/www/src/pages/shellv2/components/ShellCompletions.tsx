import React, { forwardRef } from "react";

interface ShellCompletionsProps {
    completions: string[];
    show: boolean;
    index: number;
    position: { x: number; y: number };
}

const ShellCompletions = forwardRef<HTMLUListElement, ShellCompletionsProps>(({ completions, show, index, position }, ref) => {
    if (!show) return null;

    return (
        <div style={{
            position: "absolute",
            top: position.y,
            left: position.x,
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
            <ul ref={ref} style={{ listStyle: "none", margin: 0, padding: 0 }}>
                {completions.map((c, i) => (
                    <li key={i} style={{
                        padding: "4px 8px",
                        background: i === index ? "#094771" : "transparent",
                        cursor: "pointer"
                    }}>
                        {c}
                    </li>
                ))}
            </ul>
        </div>
    );
});

export default ShellCompletions;
