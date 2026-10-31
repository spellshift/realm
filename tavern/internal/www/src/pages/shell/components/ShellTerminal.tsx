import React, { RefObject } from "react";
import { Loader2 } from "lucide-react";
import ShellCompletions from "./ShellCompletions";
import { DocTooltip } from "./DocTooltip";

interface ShellTerminalProps {
  termRef: RefObject<HTMLDivElement>;
  fontSize?: number;
  isConnecting?: boolean;
  isPromptBusy?: boolean;
  isMissedCallback?: boolean;
  promptPos?: {
    top: number;
    left: number;
    width: number;
    height: number;
    visible: boolean;
  };
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
  fontSize,
  isConnecting = false,
  isPromptBusy = false,
  isMissedCallback = false,
  promptPos,
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
      data-font-size={fontSize}
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
      {isPromptBusy && !isConnecting && (promptPos?.visible ?? true) && (
        <div
          data-testid="shell-prompt-spinner"
          role="status"
          aria-label="Commands queued or executing"
          className="absolute z-10 flex items-center justify-between bg-[#11131a] pointer-events-none select-none"
          style={{
            top: `${promptPos?.top ?? 6}px`,
            left: `${promptPos?.left ?? 6}px`,
            width: `${promptPos?.width ?? 24}px`,
            height: `${promptPos?.height ?? 18}px`,
          }}
        >
          <span className="w-2/3 flex items-center justify-start -ml-0.5">
            <Loader2
              data-testid="shell-prompt-spinner-icon"
              className={`w-3.5 h-3.5 animate-spin ${
                isMissedCallback ? "text-red-400" : "text-amber-400"
              }`}
            />
          </span>
          <span
            className="w-1/3 flex items-center justify-center text-[#e2e8f0]"
            style={{
              fontFamily:
                "'JetBrains Mono', 'Fira Code', 'SFMono-Regular', Menlo, Monaco, Consolas, monospace",
              fontSize: `${fontSize ?? 13.5}px`,
              lineHeight: 1,
            }}
          >
            &gt;
          </span>
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
        fontSize={fontSize}
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
