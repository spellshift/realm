import AlertError from "../../components/tavern-base-ui/AlertError";
import Badge from "../../components/tavern-base-ui/badge/Badge";
import Breadcrumbs from "../../components/Breadcrumbs";
import { AccessGate } from "../../components/access-gate";
import { useShell } from "./hooks/useShell";
import ShellTerminal from "./components/ShellTerminal";
import ShellCompletions from "./components/ShellCompletions";
import ShellStatusBar from "./components/ShellStatusBar";

const ShellV2 = () => {
    const {
        termRef,
        completions,
        showCompletions,
        completionIndex,
        completionPos,
        completionsListRef,
        portalId,
        timeUntilCallback,
        isMissedCallback,
        connectionError,
        loading,
        data
    } = useShell();

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
                <div className="flex items-center gap-4 mb-4">
                    <Breadcrumbs pages={[{ label: "Shell", link: window.location.pathname }]} />
                    <Badge badgeStyle={{ color: "purple" }}>Pre-alpha release</Badge>
                    <h1 className="text-xl font-bold">Eldritch Shell for {data?.node?.beacon?.name}</h1>
                </div>

                <div className="flex-grow rounded overflow-hidden relative border border-[#333]">
                    <ShellTerminal ref={termRef} />
                    <ShellCompletions
                        ref={completionsListRef}
                        completions={completions}
                        show={showCompletions}
                        index={completionIndex}
                        position={completionPos}
                    />
                </div>

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
