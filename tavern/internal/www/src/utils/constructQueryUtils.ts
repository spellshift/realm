import { sub } from "date-fns";
import { Filters } from "../context/FilterContext";
import { FilterBarOption } from "./interfacesUI";
import { getBeaconFilterNameByTypes, getTomeFilterNameByTypes } from "./utils";
import { OnlineOfflineFilterType } from "./enums";

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

    if(groups.length < 1 && services.length < 1){
      return null;
    }

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
    onlineOfflineStatus: Array<string>,
    currentTimestamp?: Date
  ){
    const tagQuery = constructTagFieldsQuery(groups, services);

    const hostStatusFilter = constructHostStatusFilter(onlineOfflineStatus, currentTimestamp);

    if(hosts.length < 1 && !tagQuery && platforms.length < 1 && primaryIP.length < 1 && !hostStatusFilter){
      return null;
    }

    return {
      "hasHostWith": {
        ...(tagQuery && {"and": constructTagFieldsQuery(groups, services)}),
        ...(hosts.length > 0) && {"nameIn": hosts},
        ...(platforms.length > 0) && {"platformIn": platforms},
        ...(primaryIP.length > 0) && {"primaryIPIn": primaryIP},
        ...(hostStatusFilter && hostStatusFilter)
      }
    }
};

export function constructBeaconFilterQuery(
  beaconFields: Array<FilterBarOption>,
  currentTimestamp?: Date
){
    const {beacon: beacons, group: groups, service: services, platform: platforms, host:hosts, principal, primaryIP, transport, onlineOfflineStatus} = getBeaconFilterNameByTypes(beaconFields);

    const beaconStatusFilter = constructBeaconStatusFilter(onlineOfflineStatus, currentTimestamp);

    const hostFiledQuery = constructHostFieldQuery(groups, services, platforms, hosts, primaryIP, onlineOfflineStatus, currentTimestamp);

    if(beacons.length < 1 && principal.length < 1 && transport.length < 1 && !beaconStatusFilter && !hostFiledQuery){
      return null;
    }

    const hasBeaconWith: any = {
      ...(beacons.length > 0 && {"nameIn": beacons}),
      ...(principal.length > 0 && {"principalIn": principal}),
      ...(transport.length > 0 && {"transportIn": transport}),
      ...hostFiledQuery
    };

    if (beaconStatusFilter) {
      Object.assign(hasBeaconWith, beaconStatusFilter);
    }

    return {
      "hasBeaconWith": hasBeaconWith
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

export function constructTaskFilterQuery(
  filter: Filters,
  currentTimestamp?: Date
){
    const beaconFilterQuery = constructBeaconFilterQuery(filter.beaconFields, currentTimestamp);
    if(!filter.taskOutput && !beaconFilterQuery){
      return null;
    }

    return {
      "hasTasksWith": {
        ...(filter.taskOutput && {"outputContains": filter.taskOutput}),
        ...(beaconFilterQuery && beaconFilterQuery),
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

export function constructHostTaskFilterQuery(
  filter: Filters,
  currentTimestamp?: Date
){
    const beaconFilterQuery = constructBeaconFilterQuery(filter.beaconFields, currentTimestamp);
    const questFilterQuery = constructQuestFilterQuery(filter);

    if(!filter.taskOutput && !beaconFilterQuery && !questFilterQuery){
      return null;
    }

    return {
      "hasTasksWith": {
        ...(questFilterQuery && {"hasQuestWith": questFilterQuery}),
        ...(filter.taskOutput && {"outputContains": filter.taskOutput}),
        ...(beaconFilterQuery && beaconFilterQuery),
      }
    };

};

const createRecentlyLostQuery = (start: Date, end: Date) => ({
  and: [
    { nextSeenAtGTE: start.toISOString() },
    { nextSeenAtLT: end.toISOString() }
  ]
});

export function constructBeaconStatusFilter(
  status: Array<string>,
  currentTimestamp?: Date
) {
  if (!currentTimestamp) return null;

  const conditions = [
    ...status.includes(OnlineOfflineFilterType.OnlineBeacons) ? [{nextSeenAtGTE: currentTimestamp.toISOString()}] : [],
    ...status.includes(OnlineOfflineFilterType.RecentlyLostBeacons) ? [createRecentlyLostQuery(sub(currentTimestamp, { minutes: 5 }), currentTimestamp)] : [],
  ]

  if (conditions.length === 0) return null;
  if (conditions.length === 1) return conditions[0];
  return { or: conditions };
}

export function constructHostStatusFilter(
  status: Array<string>,
  currentTimestamp?: Date
) {
  if (!currentTimestamp) return null;

  const conditions = [
    ...status.includes(OnlineOfflineFilterType.OfflineHost) ? [{nextSeenAtLT: currentTimestamp.toISOString()}] : [],
    ...status.includes(OnlineOfflineFilterType.RecentlyLostHost) ? [createRecentlyLostQuery(sub(currentTimestamp, { minutes: 5 }), currentTimestamp)] : [],
  ]

  if (conditions.length === 0) return null;
  if (conditions.length === 1) return conditions[0];
  return { or: conditions };
}
