import { Filters } from "../context/FilterContext";
import { FilterBarOption } from "./consts";
import { getFilterNameByTypes } from "./utils";

export function constructTagQueryFormat(
    kind: string,
    tags: Array<string>
){
    return {
        "hasTagsWith": {
            "kind": kind,
            "nameIn":  tags
        }
    }
}

export const constructTagFieldsQuery = function (
    groups: Array<string>,
    services: Array<string>
    ){

    return [
      ...(groups.length > 0) ? [constructTagQueryFormat('group', groups)] : [],
      ...(services.length > 0) ? [constructTagQueryFormat('service', services)]  : [],
    ]

};

export function constructHostFieldQuery(
    groups: Array<string>,
    services: Array<string>,
    platforms: Array<string>,
    hosts: Array<string>
  ){
    if(hosts.length < 1 && groups.length < 1 && services.length < 1 && platforms.length < 1){
      return null;
    }

    return {
      "hasHostWith": {
        "and": constructTagFieldsQuery(groups, services),
        ...(hosts.length > 0) && {"nameIn": hosts},
        ...(platforms.length > 0) && {"platformIn": platforms}
      }
    }
};

export function constructBeaconFilterQuery(beaconFields: Array<FilterBarOption>){
    const {beacon: beacons, group: groups, service: services, platform: platforms, host:hosts} = getFilterNameByTypes(beaconFields);
    const hostFiledQuery = constructHostFieldQuery(groups, services, platforms, hosts);

    if(beacons.length < 1 && !hostFiledQuery){
      return null;
    }

    return {
      "hasBeaconWith": {
          ...(beacons.length > 0 && {"nameIn": beacons}),
          ...hostFiledQuery
      }
    };

};
export function constructTaskFilterQuery(filter: Filters){
    const beaconFilterQuery = constructBeaconFilterQuery(filter.beaconFields);

    if(!filter.taskOutput && !beaconFilterQuery){
      return null;
    }

    return {
      "hasTasksWith": {
        ...(filter.taskOutput && {"outputContains": filter.taskOutput}),
        ...(beaconFilterQuery && beaconFilterQuery)
      }
    };

};
