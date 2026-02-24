import { useParams } from "react-router-dom";
import AlertError from "../../components/tavern-base-ui/AlertError";
import { AccessGate } from "../../components/access-gate";
import { useShellData } from "./hooks/useShellData";
import { useCallbackTimer } from "./hooks/useCallbackTimer";
import ShellHeader from "./components/ShellHeader";
import ShellTerminal from "./components/ShellTerminal";
import ShellStatusBar from "./components/ShellStatusBar";

const ShellV2 = () => {
    const { shellId } = useParams<{ shellId: string }>();

    const {
        shellData,
        beaconData,
        loading,
        error,
        portalId,
        setPortalId
    } = useShellData(shellId);

    const { timeUntilCallback, isMissedCallback } = useCallbackTimer(beaconData);

    // Initial validation errors
    if (!shellId) {
        return (
            <div style={{ padding: "20px" }}>
                <AlertError label="Shell Connection Failed" details="No Shell ID provided in URL." />
            </div>
        );
    }

    if (error) {
        return (
            <div style={{ padding: "20px" }}>
                <AlertError label="Shell Connection Failed" details={`Failed to load shell: ${error.message}`} />
            </div>
        );
    }

    if (!loading && !shellData?.node) {
        return (
            <div style={{ padding: "20px" }}>
                <AlertError label="Shell Connection Failed" details="Shell not found." />
            </div>
        );
    }

    if (!loading && shellData?.node?.closedAt) {
        return (
            <div style={{ padding: "20px" }}>
                <AlertError label="Shell Connection Failed" details="This shell session is closed." />
            </div>
        );
    }

    if (loading) {
        return <div style={{ padding: "20px", color: "#d4d4d4" }}>Loading Shell...</div>;
    }

    return (
        <AccessGate>
            <div className="flex flex-col h-screen p-5 bg-[#1e1e1e] text-[#d4d4d4]">
                <ShellHeader beaconName={shellData?.node?.beacon?.name} />

                <ShellTerminal
                    shellId={shellId}
                    setPortalId={setPortalId}
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
