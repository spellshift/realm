import { RotateCw } from "lucide-react";
import { usePolling } from "../../context/PollingContext";

type PollingCountdownProps = {
    variant: "full" | "minimal" | "icon-only";
};

export const PollingCountdown = ({ variant }: PollingCountdownProps) => {
    const { secondsUntilNextPoll } = usePolling();

    if (variant === "icon-only") {
        return (
            <div className="flex items-center justify-center p-2">
                <RotateCw className="h-3 w-3 text-gray-500" />
            </div>
        );
    }

    if (variant === "minimal") {
        return (
            <div className="flex flex-col items-center justify-center p-2 text-gray-500">
                <RotateCw className="h-3 w-3" />
                <span className="text-xs mt-1">{secondsUntilNextPoll}s</span>
            </div>
        );
    }

    return (
        <div className="flex items-center justify-end gap-x- p-2 gap-2 text-gray-500 text-sm border-t border-gray-800">
            <RotateCw className="h-3 w-3 shrink-0" />
            <span>Next update:</span><span className="w-7">{secondsUntilNextPoll}s</span>
        </div>
    );
};
