import React from "react";
import {
    AlertTriangle,
    Flame,
    Loader2,
    PlusCircle,
    RefreshCw,
} from "lucide-react";
import { ConnectionStatus } from "../../../lib/browser-adapter";
import { ShellTaskItem } from "../hooks/useShellData";

export interface InterpreterStatusBannerProps {
    closedAt?: string | null;
    connectionStatus: ConnectionStatus;
    connectionMessage?: string;
    isConnecting?: boolean;
    stuckTask: { task: ShellTaskItem; elapsedSeconds: number } | null;
    queuedTasks: ShellTaskItem[];
    runningTasks: ShellTaskItem[];
    timeUntilCallback: string;
    onSpawnFreshShell: () => void;
    isSpawningShell?: boolean;
    onReconnect?: () => void;
}

const truncateCmd = (cmd: string, max = 56): string => {
    const firstLine = cmd.trim().split("\n")[0] || "";
    if (firstLine.length <= max) return firstLine;
    return `${firstLine.slice(0, max)}...`;
};

const InterpreterStatusBanner: React.FC<InterpreterStatusBannerProps> = ({
    closedAt,
    connectionStatus,
    connectionMessage,
    isConnecting = false,
    stuckTask,
    onSpawnFreshShell,
    isSpawningShell = false,
    onReconnect,
}) => {
    // 1. Terminated Shell Banner
    if (closedAt) {
        return (
            <div
                data-testid="shell-closed-banner"
                className="flex flex-wrap items-center justify-between gap-2 px-3 py-2 border-b border-red-500/40 bg-red-900/30 text-red-200 text-xs"
            >
                <div className="flex items-center gap-2">
                    <AlertTriangle className="w-4 h-4 text-red-400 shrink-0" />
                    <span>
                        <strong>Shell Terminated</strong> (closed at{" "}
                        {new Date(closedAt).toLocaleTimeString()}). This transcript is read-only.
                    </span>
                </div>
                <button
                    type="button"
                    disabled={isSpawningShell}
                    onClick={onSpawnFreshShell}
                    className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded bg-red-600 hover:bg-red-500 disabled:opacity-50 text-white font-semibold transition-colors"
                >
                    {isSpawningShell ? (
                        <Loader2 className="w-3.5 h-3.5 animate-spin" />
                    ) : (
                        <PlusCircle className="w-3.5 h-3.5" />
                    )}
                    New Interpreter
                </button>
            </div>
        );
    }

    // 2. WebSocket Disconnected / Reconnecting Banner (suppressed during initial connection)
    if (!isConnecting && (connectionStatus === "disconnected" || connectionStatus === "reconnecting")) {
        return (
            <div
                data-testid="shell-ws-banner"
                className="flex items-center justify-between gap-2 px-3 py-1.5 border-b border-yellow-500/40 bg-yellow-900/30 text-yellow-200 text-xs"
            >
                <div className="flex items-center gap-2">
                    <RefreshCw className="w-3.5 h-3.5 text-yellow-400 animate-spin shrink-0" />
                    <span>
                        {connectionStatus === "reconnecting"
                            ? "Reconnecting to Tavern shell stream..."
                            : `Disconnected from shell stream${connectionMessage ? `: ${connectionMessage}` : ""}.`}
                    </span>
                </div>
                {onReconnect && (
                    <button
                        type="button"
                        onClick={onReconnect}
                        className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-yellow-500/20 hover:bg-yellow-500/30 border border-yellow-500/40 text-yellow-100 font-medium transition-colors"
                    >
                        <RefreshCw className="w-3 h-3" />
                        Reconnect Now
                    </button>
                )}
            </div>
        );
    }

    // 3. Stuck / Busy Interpreter Banner (Executing > 30s)
    if (stuckTask) {
        return (
            <div
                data-testid="stuck-interpreter-banner"
                className="flex flex-wrap items-center justify-between gap-2 px-3 py-2 border-b border-amber-500/50 bg-amber-900/30 text-amber-100 text-xs"
            >
                <div className="flex items-start sm:items-center gap-2 min-w-0">
                    <Flame className="w-4 h-4 text-amber-400 shrink-0 mt-0.5 sm:mt-0" />
                    <div className="min-w-0">
                        <span className="font-semibold text-amber-300">
                            Interpreter Busy or Potentially Stuck ({stuckTask.elapsedSeconds}s):
                        </span>{" "}
                        <code className="px-1.5 py-0.5 rounded bg-black/40 text-amber-200 font-mono">
                            {truncateCmd(stuckTask.task.input)}
                        </code>{" "}
                        <span className="text-amber-200/80">
                            is still running. Subsequent commands in this shell are queued behind it.
                        </span>
                    </div>
                </div>
                <div className="flex items-center gap-2 shrink-0">
                    <button
                        type="button"
                        disabled={isSpawningShell}
                        onClick={onSpawnFreshShell}
                        className="inline-flex items-center gap-1 px-2.5 py-1 rounded bg-amber-600 hover:bg-amber-500 disabled:opacity-50 text-white font-semibold transition-colors shadow-sm"
                    >
                        {isSpawningShell ? (
                            <Loader2 className="w-3.5 h-3.5 animate-spin" />
                        ) : (
                            <PlusCircle className="w-3.5 h-3.5" />
                        )}
                        New Interpreter
                    </button>
                </div>
            </div>
        );
    }

    return null;
};

export default InterpreterStatusBanner;
