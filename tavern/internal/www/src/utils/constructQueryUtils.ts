import { Filters } from "../context/FilterContext";
import { FilterBarOption } from "./interfacesUI";
import { getBeaconFilterNameByTypes, getTomeFilterNameByTypes } from "./utils";

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
    hosts: Array<string>,
    primaryIP: Array<string>
  ){
    if(hosts.length < 1 && groups.length < 1 && services.length < 1 && platforms.length < 1 && primaryIP.length < 1){
      return null;
    }

    return {
      "hasHostWith": {
        "and": constructTagFieldsQuery(groups, services),
        ...(hosts.length > 0) && {"nameIn": hosts},
        ...(platforms.length > 0) && {"platformIn": platforms},
        ...(primaryIP.length > 0) && {"primaryIPIn": primaryIP}
      }
    }
};

export function constructBeaconFilterQuery(beaconFields: Array<FilterBarOption>){
    const {beacon: beacons, group: groups, service: services, platform: platforms, host:hosts, principal, primaryIP} = getBeaconFilterNameByTypes(beaconFields);
    const hostFiledQuery = constructHostFieldQuery(groups, services, platforms, hosts, primaryIP);

    if(beacons.length < 1 && principal.length < 1 && !hostFiledQuery){
      return null;
    }

    return {
      "hasBeaconWith": {
          ...(beacons.length > 0 && {"nameIn": beacons}),
          ...(principal.length > 0 && {"principalIn": principal}),
          ...hostFiledQuery
      }
    };

};

export function constructTomeFilterQuery(tomeFields: Array<FilterBarOption>){
    const { Tactic, SupportModel } = getTomeFilterNameByTypes(tomeFields);

    if(Tactic.length < 1 && SupportModel.length < 1){
      return null;
    }

    return {
      "hasTomeWith": {
        "tacticIn": Tactic,
        "supportModelIn": SupportModel
      }
    };

};

export function constructTaskFilterQuery(filter: Filters){
    const tomeFilterQuery = constructTomeFilterQuery(filter.tomeFields);
    const beaconFilterQuery = constructBeaconFilterQuery(filter.beaconFields);

    if(!filter.taskOutput && !beaconFilterQuery && !tomeFilterQuery){
      return null;
    }

    return {
      // ...(tomeFilterQuery && constructTomeFilterQuery),
      "hasTasksWith": {
        ...(filter.taskOutput && {"outputContains": filter.taskOutput}),
        ...(beaconFilterQuery && beaconFilterQuery)
      }
    };

};
