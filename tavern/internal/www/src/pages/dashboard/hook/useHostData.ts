import { isAfter } from "date-fns";
import { useCallback, useEffect, useState } from "react";

import { getOfflineOnlineStatus } from "../../../utils/utils";
import { HostEdge, HostQueryTopLevel, TagEdge } from "../../../utils/interfacesQuery";

type UniqueCountHost = {
    tagId: string,
    tag: string,
    online: number,
    total: number,
    lastSeenAt: string | undefined | null
    hostsOnline: number,
    hostsTotal: number,
}

type HostUsageByKind = {
    group: Array<UniqueCountHost>,
    service: Array<UniqueCountHost>,
    platform: Array<UniqueCountHost>,
}

type UniqueCountHostByTag = {
    [key: string]: UniqueCountHost
}

const defaultHostUsage: HostUsageByKind = {
    group: [],
    service: [],
    platform: []
};

export const useHostData = (data: HostQueryTopLevel | undefined) => {
    const [loading, setLoading] = useState(false);
    const [hostActivity, setHostActivity] = useState<HostUsageByKind>(defaultHostUsage);

    const [onlineHostCount, setOnlineHostCount] = useState(0);
    const [offlineHostCount, setOfflineHostCount] = useState(0);
    const [totalHostCount, setTotalHostCount] = useState(0);

    const applyUniqueTermData = useCallback((term: string | undefined, id: string | undefined, uniqueObject: UniqueCountHostByTag, host: HostEdge, beaconStatus: any) => {
        if (!term || !id) {
            return uniqueObject;
        }

        if (term in uniqueObject) {
            const currDate = uniqueObject[term]?.lastSeenAt ? new Date(uniqueObject[term].lastSeenAt || "") : new Date("1999/08/08");
            const newDate = host?.node.lastSeenAt ? new Date(host?.node.lastSeenAt || "") : new Date("1999/07/07");;
            const replaceCallback = isAfter(newDate, currDate);

            if (replaceCallback) {
                uniqueObject[term].lastSeenAt = host.node.lastSeenAt;
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
                lastSeenAt: host.node.lastSeenAt,
                online: beaconStatus.online,
                total: beaconStatus.online + beaconStatus.offline,
                hostsOnline: beaconStatus.online > 0 ? 1 : 0,
                hostsTotal: 1
            }
        }

        return uniqueObject;
    }, []);

    const formatHostData = useCallback((hosts: HostEdge[]) => {
        const uniqueGroups: UniqueCountHostByTag = {};
        const uniqueServices: UniqueCountHostByTag = {};
        const uniquePlatform: UniqueCountHostByTag = {};

        let onlineCount = 0;
        let totalCount = 0;
        let offlineCount = 0;

        hosts?.forEach((host: HostEdge) => {
            const serviceTag = host?.node.tags?.edges ? host?.node?.tags?.edges.find((tag: TagEdge) => tag.node.kind === "service")?.node : { name: "undefined", id: "undefined" };
            const groupTag = host?.node.tags?.edges ? host?.node?.tags?.edges?.find((tag: TagEdge) => tag.node.kind === "group")?.node : { name: "undefined", id: "undefined" };
            const beaconStatus = getOfflineOnlineStatus(host?.node?.beacons?.edges || []);

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
            applyUniqueTermData(host.node.platform, host.node.platform, uniquePlatform, host, beaconStatus);
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
    }, [applyUniqueTermData]);

    useEffect(() => {
        if (data && data?.hosts?.edges && data?.hosts?.edges?.length > 0) {
            setLoading(true);
            formatHostData(data.hosts.edges);
            setLoading(false);
        }
    }, [data, formatHostData])

    return {
        loading,
        hostActivity,
        onlineHostCount,
        totalHostCount,
        offlineHostCount
    }

}
