import React, { RefObject } from "react";
import ShellCompletions from "./ShellCompletions";

interface ShellTerminalProps {
  termRef: RefObject<HTMLDivElement>;
  completions: string[];
  showCompletions: boolean;
  completionPos: { x: number; y: number };
  completionIndex: number;
}

const ShellTerminal: React.FC<ShellTerminalProps> = ({
  termRef,
  completions,
  showCompletions,
  completionPos,
  completionIndex
}) => {
  return (
    <div className="flex-grow rounded overflow-hidden relative border border-[#333]">
      <div ref={termRef} style={{ height: "100%", width: "100%" }} />
      <ShellCompletions
        completions={completions}
        show={showCompletions}
        pos={completionPos}
        index={completionIndex}
      />
    </div>
  );
};

export default ShellTerminal;
