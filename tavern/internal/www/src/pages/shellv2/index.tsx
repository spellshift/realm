import { useParams } from "react-router-dom";
import AlertError from "../../components/tavern-base-ui/AlertError";
import { AccessGate } from "../../components/access-gate";
import { useShellStatus } from "./hooks/useShellStatus";
import { useShellTerminal } from "./hooks/useShellTerminal";
import ShellHeader from "./components/ShellHeader";
import ShellTerminal from "./components/ShellTerminal";
import ShellFooter from "./components/ShellFooter";

const ShellV2 = () => {
    const { shellId } = useParams<{ shellId: string }>();

    const {
        loading,
        error,
        data,
        portalId,
        setPortalId,
        timeUntilCallback,
        isMissedCallback,
    } = useShellStatus(shellId);

    const {
        termRef,
        completions,
        showCompletions,
        completionPos,
        completionIndex,
        connectionError,
        completionsListRef,
    } = useShellTerminal(shellId, setPortalId, loading, error, data);

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
                <ShellHeader beaconName={data?.node?.beacon?.name} />

                <ShellTerminal
                    termRef={termRef}
                    completions={completions}
                    showCompletions={showCompletions}
                    completionPos={completionPos}
                    completionIndex={completionIndex}
                    completionsListRef={completionsListRef}
                />

                <ShellFooter
                    portalId={portalId}
                    timeUntilCallback={timeUntilCallback}
                    isMissedCallback={isMissedCallback}
                />
            </div>
        </AccessGate>
    );
};

export default ShellV2;
