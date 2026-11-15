import { CircleCheck, CircleX, CirclePause } from "lucide-react";
import { Ring } from "@uiball/loaders";
import { BuildTaskStatus } from "../utils/buildTasks";

// Shared status glyph for a BuildTask, used by the Built agents tab and the
// Assets table. Queued/Claimed share the paused icon; Building and the brief
// post-build "Awaiting artifact" gap share the in-progress spinner.
export const BuildStatusIcon = ({ status, size = 20 }: { status: BuildTaskStatus; size?: number }) => {
    const style = { width: size, height: size };
    switch (status) {
        case "Queued":
        case "Claimed":
            return <CirclePause style={style} className="text-gray-400 shrink-0" />;
        case "Building":
        case "Awaiting artifact":
            return (
                <span className="shrink-0 inline-flex" data-testid="build-status-spinner">
                    <Ring size={Math.round(size * 0.8)} lineWeight={2.5} speed={2} color="#7C3AED" />
                </span>
            );
        case "Built":
            return <CircleCheck style={style} className="text-green-600 shrink-0" />;
        case "Failed":
            return <CircleX style={style} className="text-red-600 shrink-0" />;
    }
};

export default BuildStatusIcon;
