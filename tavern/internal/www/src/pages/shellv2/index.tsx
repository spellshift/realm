import { useParams } from "react-router-dom";
import AlertError from "../../components/tavern-base-ui/AlertError";
import { AccessGate } from "../../components/access-gate";
import { useShellData } from "./hooks/useShellData";
import { useCallbackTimer } from "./hooks/useCallbackTimer";
import { useShellTerminal } from "./hooks/useShellTerminal";
import ShellHeader from "./components/ShellHeader";
import ShellTerminal from "./components/ShellTerminal";
import ShellStatusBar from "./components/ShellStatusBar";

const ShellV2 = () => {
    const { shellId } = useParams<{ shellId: string }>();

    const {
        loading,
        error,
        shellData,
        beaconData,
        portalData,
        portalId,
        setPortalId
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
        applyCompletion
    } = useShellTerminal(shellId, loading, error, shellData, setPortalId, isLateCheckin);

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
        <AccessGate>
            <div className="flex flex-col h-screen p-5 bg-[#1e1e1e] text-[#d4d4d4]">
                <ShellHeader shellData={shellData} />

                <ShellTerminal
                    termRef={termRef}
                    completions={completions}
                    showCompletions={showCompletions}
                    completionPos={completionPos}
                    completionIndex={completionIndex}
                    onMouseMove={handleMouseMove}
                    tooltipState={tooltipState}
                    onCompletionSelect={applyCompletion}
                />

                <ShellStatusBar
                    portalId={portalId}
                    timeUntilCallback={timeUntilCallback}
                    isMissedCallback={isMissedCallback}
                />
            </div>
        </AccessGate>
    );
};

export default ShellV2;
