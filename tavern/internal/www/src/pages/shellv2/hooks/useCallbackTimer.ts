import { useEffect, useState } from "react";
import moment from "moment";

export const useCallbackTimer = (beaconData: any) => {
    const [timeUntilCallback, setTimeUntilCallback] = useState<string>("");
    const [isMissedCallback, setIsMissedCallback] = useState(false);

    useEffect(() => {
        const updateTimer = () => {
            // @ts-ignore
            if (!beaconData?.node?.nextSeenAt) return;
            // @ts-ignore
            const next = moment(beaconData.node.nextSeenAt);
            const now = moment();
            const diff = next.diff(now, 'seconds');
            const absDiff = Math.abs(diff);

            if (diff > 1 || diff < -1) {
                const duration = moment.duration(absDiff, 'seconds');
                const hours = Math.floor(duration.asHours());
                const minutes = duration.minutes();
                const seconds = duration.seconds();

                const parts = [];
                if (hours > 0) parts.push(`${hours} hr`);
                if (minutes > 0) parts.push(`${minutes} min`);
                if (seconds > 0 || parts.length === 0) parts.push(`${seconds}s`);

                const timeStr = parts.join(" ");

                if (diff > 1) {
                    setTimeUntilCallback(`in ${timeStr}`);
                    setIsMissedCallback(false);
                } else {
                    setTimeUntilCallback(`expected ${timeStr} ago`);
                    setIsMissedCallback(true);
                }
            } else {
                setTimeUntilCallback("now");
                setIsMissedCallback(false);
            }
        };

        updateTimer(); // Initial call
        const intervalId = setInterval(updateTimer, 1000);
        return () => clearInterval(intervalId);
    }, [beaconData]);

    return { timeUntilCallback, isMissedCallback };
};
