import { BeaconType } from "../../../utils/consts";
import {add} from "date-fns";

export function getOnlineBeacons(beacons: Array<BeaconType>) : Array<BeaconType>{
    const currentDate = new Date();
    return beacons.filter((beacon: BeaconType)=> add(new Date(beacon.lastSeenAt),{seconds: beacon.interval}) >= currentDate);
}

export function isBeaconSelected(selectedBeacons: any): boolean{
    for (let key in selectedBeacons) {
        if (selectedBeacons[key] === true) {
            return true;
        }
    }
    return false;
}