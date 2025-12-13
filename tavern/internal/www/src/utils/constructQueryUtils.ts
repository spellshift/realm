import { sub } from "date-fns";
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
    primaryIP: Array<string>,
    // hostStatus: Array<string>,
    // currentTimestamp?: string
  ){
    const tagFieldQueryArray = constructTagFieldsQuery(groups, services);

    if(hosts.length < 1 && tagFieldQueryArray.length < 1 && platforms.length < 1 && primaryIP.length < 1){
      return null;
    }

    return {
      "hasHostWith": {
        ...(tagFieldQueryArray.length > 1 && {"and": constructTagFieldsQuery(groups, services)}),
        ...(hosts.length > 0) && {"nameIn": hosts},
        ...(platforms.length > 0) && {"platformIn": platforms},
        ...(primaryIP.length > 0) && {"primaryIPIn": primaryIP},
      }
    }
};

function constructBeaconStatusFilter(status: Array<string>, currentTimestamp?: Date){
    if(!currentTimestamp || status.length === 0){
      return null;
    }

    const result: Record<string, any> = {
      "or": []
    };
    if(status.length > 0)

    for(const statusValue of status){
      if(statusValue === "onlineBeacons"){
        const condition = {"nextSeenAtGTE": currentTimestamp.toISOString()}
        if(status.length > 1){
          result.or.push(condition);
        }
        else{
          return condition;
        }
      }

      if(statusValue === "offlineBeacons"){
        const condition = {"nextSeenAtLT": currentTimestamp.toISOString()}
        if(status.length > 1){
          result.or.push(condition);
        }
        else{
          return condition;
        }
      }

      if(statusValue === "recentlyLostBeacons"){
        const twoMinAgo = sub(currentTimestamp, { minutes: 5 });
        const condition = {"and": [
          {"nextSeenAtGTE": twoMinAgo.toISOString()},
          {"nextSeenAtLT": currentTimestamp.toISOString()}
        ]}

        if(status.length > 1){
          result.or.push(condition);
        }
        else{
          return condition;
        }
      }
    }

    return result;
}

export function constructBeaconFilterQuery(beaconFields: Array<FilterBarOption>, currentTimestamp?: Date){
    const {beacon: beacons, group: groups, service: services, platform: platforms, host:hosts, principal, primaryIP, beaconStatus} = getBeaconFilterNameByTypes(beaconFields);
    const hostFiledQuery = constructHostFieldQuery(groups, services, platforms, hosts, primaryIP);
    const beaconStatusFilter = constructBeaconStatusFilter(beaconStatus, currentTimestamp);

    if(beacons.length < 1 && principal.length < 1 && !hostFiledQuery && !beaconStatusFilter){
      return null;
    }

    return {
      "hasBeaconWith": {
          ...(beacons.length > 0 && {"nameIn": beacons}),
          ...(principal.length > 0 && {"principalIn": principal}),
          ...beaconStatusFilter,
          ...hostFiledQuery,
      }
    };

};

export function constructTomeFilterQuery(filter: Filters){
    const { Tactic, SupportModel } = getTomeFilterNameByTypes(filter.tomeFields);

    if(filter.tomeMultiSearch === "" && Tactic.length < 1 && SupportModel.length < 1){
      return null;
    }

    return {
      "hasTomeWith": {
        ...(filter.tomeMultiSearch && {
          "or": [
            {"paramDefsContains": filter.tomeMultiSearch},
            {"nameContains": filter.tomeMultiSearch},
            {"descriptionContains": filter.tomeMultiSearch}
          ]
        }),
        ...(Tactic.length && {"tacticIn": Tactic}),
        ...(SupportModel.length && {"supportModelIn": SupportModel})
      }
    };

};

export function constructTaskFilterQuery(filter: Filters, currentTimestamp?: Date){
    const beaconFilterQuery = constructBeaconFilterQuery(filter.beaconFields, currentTimestamp);

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

export function constructQuestFilterQuery(filter: Filters){
  const tomeFilterQuery = constructTomeFilterQuery(filter);

  if(!filter.questName && !filter.tomeMultiSearch){
    return null;
  }

  return {
      ...(filter.questName && {"nameContains": filter.questName}),
      ...(tomeFilterQuery && {
        "or": [
          {"parametersContains": filter.tomeMultiSearch},
          ...[tomeFilterQuery],
        ]
      })
    }
}

export function constructHostTaskFilterQuery(filter: Filters, currentTimestamp?: Date){
    const beaconFilterQuery = constructBeaconFilterQuery(filter.beaconFields, currentTimestamp);
    const questFilterQuery =constructQuestFilterQuery(filter);

    if(!filter.taskOutput && !beaconFilterQuery && !questFilterQuery){
      return null;
    }

    return {
      "hasTasksWith": {
        ...(questFilterQuery && {"hasQuestWith": questFilterQuery}),
        ...(filter.taskOutput && {"outputContains": filter.taskOutput}),
        ...(beaconFilterQuery && beaconFilterQuery)
      }
    };

};
