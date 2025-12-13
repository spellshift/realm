import { add } from "date-fns";
import { BeaconEdge, BeaconNode } from "./interfacesQuery";
import { PrincipalAdminTypes, TomeFilterFieldKind } from "./enums";
import { FilterBarOption, OnlineOfflineStatus, FieldInputParams, TomeFiltersByType } from "./interfacesUI";

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
    "hostStatus": Array<string>,
    "beaconStatus": Array<string>,
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
        else if (currentValue.kind === "hostStatus"){
            accumulator.hostStatus.push(currentValue.value)
        }
        else if (currentValue.kind === "beaconStatus"){
            accumulator.beaconStatus.push(currentValue.value)
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
            "hostStatus": [],
            "beaconStatus": []
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

export const OnlineOfflineOptions = [
    {
        id: "onlineBeacons",
        name: "Has online beacons",
        value: "onlineBeacons",
        label: "Has online beacons",
        kind: "beaconStatus"
    },
    {
        id: 'offlineBeacons',
        name: "Has offline beacons",
        value: 'offlineBeacons',
        label: "Has offline beacons",
        kind: "beaconStatus"
    },
    // {
    //     id: 'offlineHost',
    //     name: "Host is offline",
    //     value: "offlineHost",
    //     label: "Host is offline",
    //     kind: "hostStatus"
    // },
    {
        id: 'recentlyLostBeacons',
        name: "Recently lost beacons",
        value: "recentlyLostBeacons",
        label: "Recently lost beacons",
        kind: "beaconStatus"
    },
] as FilterBarOption[]
