import { add } from "date-fns";
import { BeaconType, FilterBarOption, QuestParam, TomeParams, TomeTag } from "./consts";

export function classNames(...classes: string[]) {
    return classes.filter(Boolean).join(' ')
}

export const convertArrayToObject = (array: Array<any>) =>
    array.reduce((acc, curr) => (acc[curr] = curr, acc), {});

export const convertArrayOfObjectsToObject = (array: Array<any>, key: string) =>
    array.reduce((acc, curr) => (acc[curr[key]] = curr, acc), {});

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

export function getFilterNameByTypes(typeFilters: Array<FilterBarOption>): {
    "beacon": Array<string>,
    "service":  Array<string>,
    "group":  Array<string>,
    "host":  Array<string>,
    "platform": Array<string>
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
        return accumulator;
    },
        {
            "beacon": [],
            "service": [],
            "group": [],
            "host": [],
            "platform": []
        });
};

export const getOfflineOnlineStatus = (beacons: any) : {online: number, offline: number} => {
    return beacons.reduce(
        (accumulator: any, currentValue: any) => {
            const beaconOffline = checkIfBeaconOffline(currentValue);
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

export function getOnlineBeacons(beacons: Array<BeaconType>): Array<BeaconType> {
    const currentDate = new Date();
    return beacons.filter((beacon: BeaconType) => add(new Date(beacon.lastSeenAt), { seconds: beacon.interval, minutes: 1 }) >= currentDate);
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
export function constructTomeParams(questParamamters?: string, tomeParameters?: string): Array<QuestParam> {
    if (!questParamamters || !tomeParameters) {
        return [];
    }

    const paramValues = JSON.parse(questParamamters) || {};
    const paramFields = JSON.parse(tomeParameters || "") || [];

    const fieldWithValue = paramFields.map((field: TomeParams) => {
        return {
            ...field,
            value: paramValues[field.name] || ""
        }
    })

    return fieldWithValue;
}
export function combineTomeValueAndFields(paramValues: { [key: string]: any }, paramFields: Array<TomeParams>): Array<QuestParam> {
    const fieldWithValue = paramFields.map((field: TomeParams) => {
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
