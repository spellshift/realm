import AlertError from "../../../components/tavern-base-ui/AlertError";
import { useShellData } from "../hooks/useShellData";
import { useCallbackTimer } from "../hooks/useCallbackTimer";
import { useShellTerminal } from "../hooks/useShellTerminal";
import ShellHeader from "./ShellHeader";
import ShellTerminal from "./ShellTerminal";
import ShellStatusBar from "./ShellStatusBar";

interface ShellSessionProps {
    shellId: string | undefined;
    onSshConnect?: (target: string, portalId: number) => void;
}

export const ShellSession = ({ shellId, onSshConnect }: ShellSessionProps) => {
    const {
        loading,
        error,
        shellData,
        beaconData,
        portalData,
        portalId,
        setPortalId,
        activeUsers
    } = useShellData(shellId);

    const { timeUntilCallback, isMissedCallback, isLateCheckin } = useCallbackTimer(beaconData);

    const {
        termRef,
        connectionError,
        completions,
        showCompletions,
        completionPos,
        completionIndex,
        handleMouseMove,
        tooltipState,
        handleCompletionSelect,
        connectionStatus,
        connectionMessage,
        handleTooltipMouseEnter,
        handleTooltipMouseLeave
    } = useShellTerminal(shellId, loading, error, shellData, setPortalId, isLateCheckin, onSshConnect, portalId);

    if (connectionError) {
        return (
            <div style={{ padding: "20px" }}>
                <AlertError label="Shell Connection Failed" details={connectionError} />
            </div>
        );
    }

    if (loading) {
        return <div style={{ padding: "20px", color: "#d4d4d4" }}>Loading Shell...</div>;
    }

    return (
        <div className="flex flex-col h-full p-5 bg-[#1e1e1e] text-[#d4d4d4]">
            <ShellHeader shellData={shellData} activeUsers={activeUsers} />

            <ShellTerminal
                termRef={termRef}
                completions={completions}
                showCompletions={showCompletions}
                completionPos={completionPos}
                completionIndex={completionIndex}
                onMouseMove={handleMouseMove}
                onMouseLeave={handleTooltipMouseLeave}
                tooltipState={tooltipState}
                handleCompletionSelect={handleCompletionSelect}
                onTooltipMouseEnter={handleTooltipMouseEnter}
                onTooltipMouseLeave={handleTooltipMouseLeave}
            />

            <ShellStatusBar
                portalId={portalId}
                timeUntilCallback={timeUntilCallback}
                isMissedCallback={isMissedCallback}
                connectionStatus={connectionStatus}
                connectionMessage={connectionMessage}
            />
        </div>
    );
};
