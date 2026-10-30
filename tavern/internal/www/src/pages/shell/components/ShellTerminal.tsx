import React, { RefObject } from "react";
import { Loader2 } from "lucide-react";
import ShellCompletions from "./ShellCompletions";
import { DocTooltip } from "./DocTooltip";

interface ShellTerminalProps {
  termRef: RefObject<HTMLDivElement>;
  isConnecting?: boolean;
  completions: string[];
  showCompletions: boolean;
  completionPos: { x: number; y: number };
  completionIndex: number;
  onMouseMove: (e: React.MouseEvent<HTMLDivElement>) => void;
  onMouseLeave?: (e: React.MouseEvent<HTMLDivElement>) => void;
  tooltipState: {
    visible: boolean;
    x: number;
    y: number;
    signature: string;
    description: string;
  };
  handleCompletionSelect: (completion: string) => void;
  onTooltipMouseEnter?: () => void;
  onTooltipMouseLeave?: () => void;
}

const ShellTerminal: React.FC<ShellTerminalProps> = ({
  termRef,
  isConnecting = false,
  completions,
  showCompletions,
  completionPos,
  completionIndex,
  onMouseMove,
  onMouseLeave,
  tooltipState,
  handleCompletionSelect,
  onTooltipMouseEnter,
  onTooltipMouseLeave
}) => {
  return (
    <div
      data-tour="shell-terminal"
      className="flex-grow overflow-hidden relative bg-[#11131a]"
    >
      {isConnecting && (
        <div
          data-testid="shell-terminal-loading"
          role="status"
          aria-label="Connecting to shell websocket"
          className="absolute inset-0 z-20 flex items-center justify-center bg-[#11131a]"
        >
          <Loader2 className="w-10 h-10 text-gray-400 animate-spin" />
        </div>
      )}
      <div
        ref={termRef}
        className={`p-1.5 ${isConnecting ? "invisible" : ""}`}
        style={{ height: "100%", width: "100%" }}
        onMouseMove={onMouseMove}
        onMouseLeave={onMouseLeave}
      />
      <ShellCompletions
        completions={completions}
        show={showCompletions}
        pos={completionPos}
        index={completionIndex}
        onCompletionSelect={handleCompletionSelect}
      />
      <DocTooltip
          signature={tooltipState.signature}
          description={tooltipState.description}
          x={tooltipState.x}
          y={tooltipState.y}
          visible={tooltipState.visible}
          onMouseEnter={onTooltipMouseEnter}
          onMouseLeave={onTooltipMouseLeave}
      />
    </div>
  );
};

export default ShellTerminal;
