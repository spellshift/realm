import { add } from "date-fns";
import { BeaconEdge, BeaconNode } from "./interfacesQuery";
import { OnlineOfflineFilterType, PageNavItem, PrincipalAdminTypes, TomeFilterFieldKind } from "./enums";
import { FilterBarOption, OnlineOfflineStatus, FieldInputParams, TomeFiltersByType } from "./interfacesUI";

const pathToNavItem: Record<string, PageNavItem> = {
    '/dashboard': PageNavItem.dashboard,
    '/quests': PageNavItem.quests,
    '/tasks': PageNavItem.tasks,
    '/createQuest': PageNavItem.createQuest,
    '/hosts': PageNavItem.hosts,
    '/tomes': PageNavItem.tomes,
    '/assets': PageNavItem.assets,
    '/admin': PageNavItem.admin,
};

export function getNavItemFromPath(pathname: string): PageNavItem {
    // Check for exact match first
    if (pathToNavItem[pathname]) {
        return pathToNavItem[pathname];
    }
    // Check for prefix matches (e.g., /hosts/:hostId -> hosts)
    if (pathname.startsWith('/hosts/')) return PageNavItem.hosts;
    if (pathname.startsWith('/tasks/')) return PageNavItem.tasks;
    if (pathname.startsWith('/shells/')) return PageNavItem.hosts;

    return PageNavItem.dashboard;
}

/**
 * Returns true if we're on a host detail page (e.g., /hosts/:hostId)
 * vs the host list page (/hosts)
 */
export function isHostDetailPath(pathname: string): boolean {
    return pathname.startsWith('/hosts/');
}

export function classNames(...classes: string[]) {
    return classes.filter(Boolean).join(' ')
}

export const convertArrayToObject = (array: Array<any>) =>
    array.reduce((acc, curr) => (acc[curr] = curr, acc), {});

export const safelyJsonParse = (value: string) => {
    let error = false;
    let params = [];
    if (value !== "") {
        try {
            params = JSON.parse(value);
        }
        catch {
            error = true;
        }
    }
    return { error, params };
};

export function getBeaconFilterNameByTypes(typeFilters: Array<FilterBarOption>): {
    "beacon": Array<string>,
    "service":  Array<string>,
    "group":  Array<string>,
    "host":  Array<string>,
    "platform": Array<string>,
    "principal": Array<string>,
    "primaryIP": Array<string>,
    "transport": Array<string>,
    "onlineOfflineStatus": Array<string>
} {
    return typeFilters.reduce((accumulator: any, currentValue: any) => {
        if (currentValue.kind === "beacon") {
            accumulator.beacon.push(currentValue.name);
        }
        else if (currentValue.kind === "platform") {
            accumulator.platform.push(currentValue.name);
        }
        else if (currentValue.kind === "service") {
            accumulator.service.push(currentValue.name);
        }
        else if (currentValue.kind === "group") {
            accumulator.group.push(currentValue.name);
        }
        else if (currentValue.kind === "host") {
            accumulator.host.push(currentValue.name);
        }
        else if (currentValue.kind === "principal"){
            accumulator.principal.push(currentValue.name);
        }
        else if (currentValue.kind === "primaryIP"){
            accumulator.primaryIP.push(currentValue.name);
        }
        else if (currentValue.kind === "transport"){
            accumulator.transport.push(currentValue.name);
        }
        else if (currentValue.kind === "onlineOfflineStatus"){
            accumulator.onlineOfflineStatus.push(currentValue.value);
        }
        return accumulator;
    },
        {
            "beacon": [],
            "service": [],
            "group": [],
            "host": [],
            "platform": [],
            "principal": [],
            "primaryIP": [],
            "transport": [],
            "onlineOfflineStatus": []
        });
};

export function getTomeFilterNameByTypes(typeFilters: Array<FilterBarOption>): TomeFiltersByType {
    return typeFilters.reduce((accumulator: any, currentValue: any) => {
        if (currentValue.kind === TomeFilterFieldKind.SupportModel) {
            accumulator[TomeFilterFieldKind.SupportModel].push(currentValue.value);
        }
        else if (currentValue.kind === TomeFilterFieldKind.Tactic) {
            accumulator[TomeFilterFieldKind.Tactic].push(currentValue.value);
        }
        return accumulator;
    },
        {
            [TomeFilterFieldKind.SupportModel]: [],
            [TomeFilterFieldKind.Tactic]: []
        });
};

export const getFormatForPrincipal = (beacons: BeaconEdge[]) => {
    const uniqueListOFPrincipals = beacons.reduce((acc: any, curr: BeaconEdge) => (acc[curr.node["principal"]] = curr, acc), {});
    const princialUserList = Object.values(PrincipalAdminTypes) as Array<string>;
    const finalList = [] as Array<string>;
    for (const property in uniqueListOFPrincipals) {
        if(princialUserList.indexOf(property) !== -1){
            finalList.unshift(property);
        }
        else{
            finalList.push(property);
        }
    }
    return finalList;
};


export const getOfflineOnlineStatus = (beacons: BeaconEdge[]) : OnlineOfflineStatus => {
    return beacons.reduce(
        (accumulator: OnlineOfflineStatus, currentValue: BeaconEdge) => {
            const beaconOffline = checkIfBeaconOffline(currentValue.node);
            if (beaconOffline) {
                accumulator.offline += 1;
            }
            else {
                accumulator.online += 1;
            }
            return accumulator;
        },
        {
            online: 0,
            offline: 0
        },
    );
};

export function getOnlineBeacons(beacons: Array<BeaconNode>): Array<BeaconNode> {
    const currentDate = new Date();
    return beacons.filter((beacon: BeaconNode) => add(new Date(beacon.lastSeenAt), { seconds: beacon.interval, minutes: 1 }) >= currentDate);
}
export function checkIfBeaconOffline(beacon: { lastSeenAt: string, interval: number }): boolean {
    const currentDate = new Date();
    return add(new Date(beacon?.lastSeenAt), { seconds: beacon?.interval, minutes: 1 }) < currentDate;
}

export function isBeaconSelected(selectedBeacons: any): boolean {
    for (let key in selectedBeacons) {
        if (selectedBeacons[key] === true) {
            return true;
        }
    }
    return false;
}

export function getTacticColor(tactic: string) {
    switch (tactic) {
        case "RECON":
            return "#22c55e";
        case "RESOURCE_DEVELOPMENT":
            return "#f97316";
        case "INITIAL_ACCESS":
            return "#ef4444";
        case "EXECUTION":
            return "#a855f7";
        case "PERSISTENCE":
            return "#1e40af";
        case "PRIVILEGE_ESCALATION":
            return "#9f1239";
        case "DEFENSE_EVASION":
            return "#2dd4bf";
        case "CREDENTIAL_ACCESS":
            return "#020617";
        case "DISCOVERY":
            return "#60a5fa";
        case "LATERAL_MOVEMENT":
            return "#3b0764";
        case "COMMAND_AND_CONTROL":
            return "#facc15";
        case "EXFILTRATION":
            return "#f9a8d4";
        case "IMPACT":
            return "#d946ef";
        case "UNSPECIFIED":
        default:
            return "#4b5563";
    }
}
export function constructTomeParams(questParamamters?: string | null, tomeParameters?: string | null): Array<FieldInputParams> {
    if (!questParamamters || !tomeParameters) {
        return [];
    }

    const paramValues = JSON.parse(questParamamters) || {};
    const paramFields = JSON.parse(tomeParameters || "") || [];

    const fieldWithValue = paramFields.map((field: FieldInputParams) => {
        return {
            ...field,
            value: paramValues[field.name] || ""
        }
    })

    return fieldWithValue;
}
export function combineTomeValueAndFields(paramValues: { [key: string]: any }, paramFields: Array<FieldInputParams>): Array<FieldInputParams> {
    const fieldWithValue = paramFields.map((field: FieldInputParams) => {
        return {
            ...field,
            value: paramValues[field.name] || ""
        }
    })

    return fieldWithValue;
}

export function groupBy<T>(collection: T[], key: keyof T): { [key: string]: T[] } {
    const groupedResult = collection.reduce((previous, current) => {
        if (!previous[current[key]]) {
            previous[current[key]] = [] as T[];
        }

        previous[current[key]].push(current);
        return previous;
    }, {} as any);
    return groupedResult
}

export const mapEnumToUIOptionField = (enumObj: Record<string, string>, kind: string) => {
    return Object.entries(enumObj).map(([key, value]) => ({
        id: key,
        name: value,
        value: key,
        label: value,
        kind: kind
    }));
};

export function getEnumKey<T extends Record<string, string>>(
    enumObj: T,
    value: string | undefined | null
): string {
    if (!value) {
        return "";
    }

    const key = Object.keys(enumObj).find(k => enumObj[k] === value);
    return key || value;
}

export const OnlineOfflineOptions = [
    {
        id: OnlineOfflineFilterType.OnlineBeacons,
        name: "Online beacons",
        value: OnlineOfflineFilterType.OnlineBeacons,
        label: "Online beacons",
        kind: "onlineOfflineStatus"
    },
    {
        id: OnlineOfflineFilterType.OfflineHost,
        name: "Offline hosts",
        value:  OnlineOfflineFilterType.OfflineHost,
        label: "Offline hosts",
        kind: "onlineOfflineStatus"
    },
    {
        id: OnlineOfflineFilterType.RecentlyLostHost,
        name: "Recently lost hosts",
        value: OnlineOfflineFilterType.RecentlyLostHost,
        label: "Recently lost hosts",
        kind: "onlineOfflineStatus"
    },
    {
        id: OnlineOfflineFilterType.RecentlyLostBeacons,
        name: "Recently lost beacons",
        value: OnlineOfflineFilterType.RecentlyLostBeacons,
        label: "Recently lost beacons",
        kind: "onlineOfflineStatus"
    },
] as FilterBarOption[];

export const API_ENDPOINT = process.env.REACT_APP_API_ENDPOINT ?? 'http://localhost:8000';

export const formatBytes = (bytes: number, decimals = 2) => {
    if (!+bytes) return '0 Bytes';
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['Bytes', 'KiB', 'MiB', 'GiB', 'TiB', 'PiB', 'EiB', 'ZiB', 'YiB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
}
