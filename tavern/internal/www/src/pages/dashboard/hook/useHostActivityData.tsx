import { isAfter } from "date-fns";
import { useCallback, useEffect, useState } from "react";

import { HostType, TomeTag } from "../../../utils/consts";
import { getOfflineOnlineStatus } from "../../../utils/utils";

type UniqueCountHost = {
    tagId: string,
    tag: string,
    online: number,
    total: number,
    lastSeenAt: string | undefined | null
    hostsOnline: number,
    hostsTotal: number,
}

type HostUsageByKindProps = {
    group: Array<UniqueCountHost>,
    service: Array<UniqueCountHost>,
    platform: Array<UniqueCountHost>,
}

type UniqueCountHostByTag = {
    [key: string]: UniqueCountHost
}

const defaultHostUsage = {
    group: [],
    service: [],
    platform: []
} as HostUsageByKindProps;


export const useHostAcitvityData = (data: Array<HostType>) => {
    const [loading, setLoading] = useState(false);
    const [hostActivity, setHostActivity] = useState<any>(defaultHostUsage);

    const [onlineHostCount, setOnlineHostCount] = useState(0);
    const [offlineHostCount, setOfflineHostCount] = useState(0);
    const [totalHostCount, setTotalHostCount] = useState(0);

    const applyUniqueTermData = useCallback((term: string | undefined, id: string | undefined, uniqueObject: UniqueCountHostByTag, host: HostType, beaconStatus: any) => {
        if (!term || !id) {
            return uniqueObject;
        }

        if (term in uniqueObject) {
            const currDate = uniqueObject[term]?.lastSeenAt ? new Date(uniqueObject[term].lastSeenAt || "") : new Date("1999/08/08");
            const newDate = host?.lastSeenAt ? new Date(host?.lastSeenAt || "") : new Date("1999/07/07");;
            const replaceCallback = isAfter(newDate, currDate);

            if (replaceCallback) {
                uniqueObject[term].lastSeenAt = host.lastSeenAt;
            }
            uniqueObject[term].total += beaconStatus.online + beaconStatus.offline;
            uniqueObject[term].online += beaconStatus.online;

            if (beaconStatus.online > 0) {
                uniqueObject[term].hostsOnline += 1;
            }
            uniqueObject[term].hostsTotal += 1;

        }
        else {
            uniqueObject[term] = {
                tagId: id,
                tag: term,
                lastSeenAt: host.lastSeenAt,
                online: beaconStatus.online,
                total: beaconStatus.online + beaconStatus.offline,
                hostsOnline: beaconStatus.online > 0 ? 1 : 0,
                hostsTotal: 1
            }
        }

        return uniqueObject;
    }, []);

    const getformattedHosts = useCallback((hosts: any) => {
        const uniqueGroups = {};
        const uniqueServices = {};
        const uniquePlatform = {};

        let onlineCount = 0;
        let totalCount = 0;
        let offlineCount = 0;

        hosts?.forEach((host: HostType) => {
            const serviceTag = host?.tags ? host?.tags.find((tag: TomeTag) => tag.kind === "service") : { name: "undefined", id: "undefined" };
            const groupTag = host?.tags ? host?.tags?.find((tag: TomeTag) => tag.kind === "group") : { name: "undefined", id: "undefined" };
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

            applyUniqueTermData(groupTag?.name, groupTag?.id, uniqueGroups, host, beaconStatus);
            applyUniqueTermData(serviceTag?.name, serviceTag?.id, uniqueServices, host, beaconStatus);
            applyUniqueTermData(host.platform, host.platform, uniquePlatform, host, beaconStatus);
        });

        setHostActivity(
            {
                "group": Object.values(uniqueGroups),
                "service": Object.values(uniqueServices),
                "platform": Object.values(uniquePlatform)
            }
        );

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
