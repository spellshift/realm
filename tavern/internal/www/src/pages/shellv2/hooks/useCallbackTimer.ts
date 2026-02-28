import { useEffect, useState } from "react";
import moment from "moment";

export const useCallbackTimer = (beaconData: any) => {
    const [timeUntilCallback, setTimeUntilCallback] = useState<string>("");
    const [isMissedCallback, setIsMissedCallback] = useState(false);
    const [isLateCheckin, setIsLateCheckin] = useState(false);

    useEffect(() => {
        const updateTimer = () => {
            // @ts-ignore
            if (!beaconData?.node?.nextSeenAt) return;
            // @ts-ignore
            const next = moment(beaconData.node.nextSeenAt);
            const now = moment();
            const diff = next.diff(now, 'seconds');

            if (diff > 1 || diff < -1) {
                if (diff > 1) {
                    setTimeUntilCallback(next.fromNow());
                    setIsMissedCallback(false);
                    setIsLateCheckin(false);
                } else {
                    // next.fromNow() returns "X [time_unit] ago" for past dates
                    // We want "expected X [time_unit] ago"
                    // However, fromNow() already includes "ago", so just prepend "expected "
                    // But wait, "expected 2 minutes ago" vs "expected 2 minutes ago ago"? No, fromNow() returns "2 minutes ago".
                    // So `expected ${next.fromNow()}` yields "expected 2 minutes ago". Correct.
                    setTimeUntilCallback(`expected ${next.fromNow()}`);
                    setIsMissedCallback(true);

                    if (diff < -300) {
                        setIsLateCheckin(true);
                    } else {
                        setIsLateCheckin(false);
                    }
                }
            } else {
                setTimeUntilCallback("now");
                setIsMissedCallback(false);
                setIsLateCheckin(false);
            }
        };

        updateTimer(); // Initial call
        const intervalId = setInterval(updateTimer, 1000);
        return () => clearInterval(intervalId);
    }, [beaconData]);

    return { timeUntilCallback, isMissedCallback, isLateCheckin };
};
