import { useEffect, useState } from "react";
import moment from "moment";

export const useCallbackTimer = (beaconData: any) => {
    const [timeUntilCallback, setTimeUntilCallback] = useState<string>("");
    const [isMissedCallback, setIsMissedCallback] = useState(false);

    useEffect(() => {
        const updateTimer = () => {
            if (!beaconData?.node?.nextSeenAt) return;
            const next = moment(beaconData.node.nextSeenAt);
            const now = moment();
            const diff = next.diff(now, 'seconds');

            if (diff > 0) {
                setTimeUntilCallback(`in ${diff} seconds`);
                setIsMissedCallback(false);
            } else {
                const missedSeconds = Math.abs(diff);
                setTimeUntilCallback(`expected ${missedSeconds} seconds ago`);
                setIsMissedCallback(true);
            }
        };

        updateTimer(); // Initial call
        const intervalId = setInterval(updateTimer, 1000);
        return () => clearInterval(intervalId);
    }, [beaconData]);

    return { timeUntilCallback, isMissedCallback };
};
