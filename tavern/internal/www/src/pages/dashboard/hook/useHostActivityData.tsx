import { isAfter } from "date-fns";
import { useCallback, useEffect, useState } from "react";

import { HostType, TomeTag } from "../../../utils/consts";
import { getOfflineOnlineStatus } from "../../../utils/utils";

type UniqueCountHost = {
    tagId: string,
    group: string,
    online: number,
    total: number,
    lastSeenAt: string | undefined | null
}

type UniqueCountHostByGroup = {
    [key: string]: UniqueCountHost
}


export const useHostAcitvityData = (data: Array<HostType>) => {
    const [loading, setLoading] = useState(false);
    const [hostActivity, setHostActivity] = useState<Array<UniqueCountHost>>([]);
    const [onlineHostCount, setOnlineHostCount] = useState(0);
    const [offlineHostCount, setOfflineHostCount] = useState(0);
    const [totalHostCount, setTotalHostCount] = useState(0);

    const getformattedHosts = useCallback((hosts: any) => {
        const uniqueGroups = {} as UniqueCountHostByGroup;
        let onlineCount = 0;
        let totalCount = 0;
        let offlineCount = 0;

        hosts?.forEach((host: HostType) => {
            const groupTag = host?.tags && host?.tags.find((tag: TomeTag) => tag.kind === "group");
            const beaconStatus = getOfflineOnlineStatus(host.beacons || []);

            if (beaconStatus.online > 0) {
                onlineCount += 1;
            }
            else {
                offlineCount += 1;
            }

            if (beaconStatus.online > 0 || beaconStatus.offline > 0) {
                totalCount += 1;
            }

            if (groupTag) {
                const groupName = groupTag.name
                if (groupName in uniqueGroups) {
                    const currDate = uniqueGroups[groupName]?.lastSeenAt ? new Date(uniqueGroups[groupName].lastSeenAt || "") : new Date("1999/08/08");
                    const newDate = host?.lastSeenAt ? new Date(host?.lastSeenAt || "") : new Date("1999/07/07");;
                    const replaceCallback = isAfter(newDate, currDate);

                    if (replaceCallback) {
                        uniqueGroups[groupName].lastSeenAt = host.lastSeenAt;
                    }
                    uniqueGroups[groupName].total += beaconStatus.online + beaconStatus.offline;
                    uniqueGroups[groupName].online += beaconStatus.online;
                }
                else {
                    uniqueGroups[groupName] = {
                        tagId: groupTag.id,
                        group: groupTag.name,
                        lastSeenAt: host.lastSeenAt,
                        online: beaconStatus.online,
                        total: beaconStatus.online + beaconStatus.offline
                    }
                }
            }

        });

        setHostActivity(Object.values(uniqueGroups));
        setOnlineHostCount(onlineCount);
        setTotalHostCount(totalCount);
        setOfflineHostCount(offlineCount);
    }, []);

    useEffect(() => {
        if (data && data?.length > 0) {
            setLoading(true);
            getformattedHosts(data);
            setLoading(false);
        }
    }, [data, getformattedHosts])

    return {
        loading,
        hostActivity,
        onlineHostCount,
        totalHostCount,
        offlineHostCount
    }

}
