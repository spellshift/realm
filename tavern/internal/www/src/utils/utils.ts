import {add} from "date-fns";
import { BeaconType } from "./consts";

export const safelyJsonParse = (value: string) => {
    let error = false;
    let params = [];
    if(value !== ""){
        try{
            params = JSON.parse(value);
        }
        catch{
            error = true;
        }
    }
    return {error, params};
};

export function getFilterNameByTypes(typeFilters: Array<any>){
    return typeFilters.reduce((accumulator:any, currentValue:any) => {
        if(currentValue.kind === "beacon"){
            accumulator.beacon.push(currentValue.name);
        }
        else if(currentValue.kind === "platform"){
            accumulator.platform.push(currentValue.name);
        }
        else if(currentValue.kind === "service"){
            accumulator.service.push(currentValue.name);
        }
        else if(currentValue.kind === "group"){
            accumulator.group.push(currentValue.name);
        }
        return accumulator;
    },
    {
        "beacon": [],
        "service": [],
        "group": [],
        "platform": []
    });
};

export function getOnlineBeacons(beacons: Array<BeaconType>) : Array<BeaconType>{
    const minIntervalBuffer = 60;
    const currentDate = new Date();
    return beacons.filter((beacon: BeaconType)=> add(new Date(beacon.lastSeenAt),{seconds: beacon.interval + minIntervalBuffer}) >= currentDate);
}
export function checkIfBeaconOnline(beacon: BeaconType) : boolean{
    const minIntervalBuffer = 60;
    const currentDate = new Date();
    return add(new Date(beacon.lastSeenAt),{seconds: beacon.interval + minIntervalBuffer}) >= currentDate;
}

export function isBeaconSelected(selectedBeacons: any): boolean{
    for (let key in selectedBeacons) {
        if (selectedBeacons[key] === true) {
            return true;
        }
    }
    return false;
}