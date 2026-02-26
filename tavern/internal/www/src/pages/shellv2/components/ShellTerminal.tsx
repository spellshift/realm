import React, { RefObject } from "react";
import ShellCompletions from "./ShellCompletions";
import { DocTooltip } from "./DocTooltip";

interface ShellTerminalProps {
  termRef: RefObject<HTMLDivElement>;
  completions: string[];
  showCompletions: boolean;
  completionPos: { x: number; y: number };
  completionIndex: number;
  onMouseMove: (e: React.MouseEvent<HTMLDivElement>) => void;
  tooltipState: {
    visible: boolean;
    x: number;
    y: number;
    signature: string;
    description: string;
  };
  onCompletionSelect: (completion: string) => void;
}

const ShellTerminal: React.FC<ShellTerminalProps> = ({
  termRef,
  completions,
  showCompletions,
  completionPos,
  completionIndex,
  onMouseMove,
  tooltipState,
  onCompletionSelect
}) => {
  return (
    <div className="flex-grow rounded overflow-hidden relative border border-[#333]">
      <div
        ref={termRef}
        style={{ height: "100%", width: "100%" }}
        onMouseMove={onMouseMove}
      />
      <ShellCompletions
        completions={completions}
        show={showCompletions}
        pos={completionPos}
        index={completionIndex}
        onSelect={onCompletionSelect}
      />
      <DocTooltip
          signature={tooltipState.signature}
          description={tooltipState.description}
          x={tooltipState.x}
          y={tooltipState.y}
          visible={tooltipState.visible}
      />
    </div>
  );
};

export default ShellTerminal;
