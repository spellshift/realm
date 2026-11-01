import { sub } from "date-fns";
import { Filters } from "../context/FilterContext";
import { FilterBarOption, HostFilterRow } from "./interfacesUI";
import { getBeaconFilterNameByTypes, getTomeFilterNameByTypes } from "./utils";
import { OnlineOfflineFilterType } from "./enums";

export function constructTagQueryFormat(
  kind: string,
  tags: Array<string>
) {
  return {
    "hasTagsWith": {
      "kind": kind,
      "nameIn": tags
    }
  }
}

export const constructTagFieldsQuery = function (
  groups: Array<string>,
  services: Array<string>
) {

  if (groups.length < 1 && services.length < 1) {
    return null;
  }

  return [
    ...(groups.length > 0) ? [constructTagQueryFormat('group', groups)] : [],
    ...(services.length > 0) ? [constructTagQueryFormat('service', services)] : [],
  ]

};

export function constructHostFieldQuery(
  groups: Array<string>,
  services: Array<string>,
  platforms: Array<string>,
  hosts: Array<string>,
  primaryIP: Array<string>,
  externalIP: Array<string>,
  onlineOfflineStatus: Array<string>,
  currentTimestamp?: Date
) {
  const tagQuery = constructTagFieldsQuery(groups, services);

  const hostStatusFilter = constructHostStatusFilter(onlineOfflineStatus, currentTimestamp);

  if (hosts.length < 1 && !tagQuery && platforms.length < 1 && primaryIP.length < 1 && externalIP.length < 1 && !hostStatusFilter) {
    return null;
  }


  return {
    "hasHostWith": {
      ...(tagQuery && { "and": constructTagFieldsQuery(groups, services) }),
      ...(hosts.length > 0) && { "idIn": hosts },
      ...(platforms.length > 0) && { "platformIn": platforms },
      ...(primaryIP.length > 0) && { "primaryIPIn": primaryIP },
      ...(externalIP.length > 0) && { "externalIPIn": externalIP },
      ...(hostStatusFilter && hostStatusFilter)
    }
  }
};

export function constructBeaconFilterQuery(
  beaconFields: Array<FilterBarOption>,
  currentTimestamp?: Date
) {
  const { beacon: beacons, group: groups, service: services, platform: platforms, host: hosts, principal, primaryIP, externalIP, transport, onlineOfflineStatus } = getBeaconFilterNameByTypes(beaconFields);

  const beaconStatusFilter = constructBeaconStatusFilter(onlineOfflineStatus, currentTimestamp);

  const hostFiledQuery = constructHostFieldQuery(groups, services, platforms, hosts, primaryIP, externalIP, onlineOfflineStatus, currentTimestamp);

  if (beacons.length < 1 && principal.length < 1 && transport.length < 1 && !beaconStatusFilter && !hostFiledQuery) {
    return null;
  }

  const hasBeaconWith: any = {
    ...(beacons.length > 0 && { "nameIn": beacons }),
    ...(principal.length > 0 && { "principalIn": principal }),
    ...(transport.length > 0 && { "transportIn": transport }),
    ...hostFiledQuery
  };

  if (beaconStatusFilter) {
    Object.assign(hasBeaconWith, beaconStatusFilter);
  }

  return {
    "hasBeaconWith": hasBeaconWith
  };

};

// Hosts page modular search bar: converts Field + Predicate + Value rows into a
// HostWhereInput. Rows AND together; multi-value predicates OR within the row
// via *In lists. Status rows reuse the same time-derived helpers as the legacy
// dropdown so online/offline/recently-lost keep their exact meaning.
export function constructHostFilterQuery(
  rows: Array<HostFilterRow>,
  currentTimestamp?: Date
) {
  const { hostConds, beaconConds } = collectRowConditions(rows, currentTimestamp, true);

  if (hostConds.length < 1 && beaconConds.length < 1) {
    return null;
  }

  const where: Record<string, any> = {};
  if (hostConds.length === 1) {
    Object.assign(where, hostConds[0]);
  } else if (hostConds.length > 1) {
    where.and = hostConds;
  }
  if (beaconConds.length === 1) {
    where.hasBeaconsWith = beaconConds[0];
  } else if (beaconConds.length > 1) {
    where.hasBeaconsWith = { and: beaconConds };
  }
  return where;
}

// Quest modal beacon picker: same Field + Predicate + Value rows, but rooted
// at BeaconWhereInput with host filters nested under hasHostWith. Status rows
// are skipped here: the picker enforces online-only separately.
export function constructBeaconIdsFilterQuery(rows: Array<HostFilterRow>) {
  const { hostConds, beaconConds } = collectRowConditions(rows, undefined, false);

  if (hostConds.length < 1 && beaconConds.length < 1) {
    return null;
  }

  const where: Record<string, any> = {};
  if (beaconConds.length === 1) {
    Object.assign(where, beaconConds[0]);
  } else if (beaconConds.length > 1) {
    where.and = beaconConds;
  }
  if (hostConds.length === 1) {
    where.hasHostWith = hostConds[0];
  } else if (hostConds.length > 1) {
    where.hasHostWith = { and: hostConds };
  }
  return where;
}

function collectRowConditions(
  rows: Array<HostFilterRow>,
  currentTimestamp: Date | undefined,
  includeStatus: boolean
): { hostConds: Array<Record<string, any>>; beaconConds: Array<Record<string, any>> } {
  const hostConds: Array<Record<string, any>> = [];
  const beaconConds: Array<Record<string, any>> = [];

  for (const row of rows) {
    if (row.field === 'status' && !includeStatus) {
      continue;
    }
    const cond = buildHostFilterRowCondition(row, currentTimestamp);
    if (!cond) {
      continue;
    }
    if (cond.host) {
      hostConds.push(cond.host);
    }
    if (cond.beacon) {
      beaconConds.push(cond.beacon);
    }
  }

  return { hostConds, beaconConds };
}

interface SplitCondition {
  host?: Record<string, any>;
  beacon?: Record<string, any>;
}

function buildHostFilterRowCondition(
  row: HostFilterRow,
  currentTimestamp?: Date
): SplitCondition | null {
  const { field, predicate, values } = row;

  switch (field) {
    case 'platform':
      return toHostCond(exactCondition('platform', predicate, values));
    case 'primaryIP':
      return toHostCond(freeTextCondition('primaryIP', predicate, values));
    case 'externalIP':
      return toHostCond(freeTextCondition('externalIP', predicate, values));
    case 'service':
    case 'group': {
      const inner = exactCondition('name', predicate, values);
      return toHostCond(inner ? { hasTagsWith: { kind: field, ...inner } } : null);
    }
    case 'host': {
      if (predicate === 'contains' || predicate === 'does-not-contain') {
        return toHostCond(freeTextCondition('name', predicate, values));
      }
      return toHostCond(exactCondition('id', predicate, values));
    }
    case 'transport':
      return toBeaconCond(exactCondition('transport', predicate, values));
    case 'principal':
      return toBeaconCond(freeTextCondition('principal', predicate, values));
    case 'beacon':
      return toBeaconCond(freeTextCondition('name', predicate, values));
    case 'status':
      return statusCondition(predicate, values, currentTimestamp);
    default:
      return null;
  }
}

function toHostCond(cond: Record<string, any> | null): SplitCondition | null {
  return cond ? { host: cond } : null;
}

function toBeaconCond(cond: Record<string, any> | null): SplitCondition | null {
  return cond ? { beacon: cond } : null;
}

// `is` / `is not` use a single value; `is in` / `is not in` OR across values.
// Mirrors the legacy dropdown output, which always used *In predicates.
function exactCondition(
  base: string,
  predicate: HostFilterRow['predicate'],
  values: string[]
): Record<string, any> | null {
  const [first] = values;
  switch (predicate) {
    case 'is':
      return first !== undefined ? { [`${base}In`]: [first] } : null;
    case 'is-not':
      return first !== undefined ? { [`${base}NotIn`]: [first] } : null;
    case 'is-in':
      return values.length > 0 ? { [`${base}In`]: [...values] } : null;
    case 'is-not-in':
      return values.length > 0 ? { [`${base}NotIn`]: [...values] } : null;
    default:
      return null;
  }
}

// Exact predicates plus substring match. The schema has no *NotContains
// predicate, so negation wraps in `not`.
function freeTextCondition(
  base: string,
  predicate: HostFilterRow['predicate'],
  values: string[]
): Record<string, any> | null {
  const exact = exactCondition(base, predicate, values);
  if (exact) {
    return exact;
  }
  const [first] = values;
  if (predicate === 'contains') {
    return first !== undefined ? { [`${base}Contains`]: first } : null;
  }
  if (predicate === 'does-not-contain') {
    return first !== undefined ? { not: { [`${base}Contains`]: first } } : null;
  }
  return null;
}

const BEACON_STATUS_VALUES: string[] = [
  OnlineOfflineFilterType.OnlineBeacons,
  OnlineOfflineFilterType.RecentlyLostBeacons,
];

const HOST_STATUS_VALUES: string[] = [
  OnlineOfflineFilterType.OfflineHost,
  OnlineOfflineFilterType.RecentlyLostHost,
];

function statusCondition(
  predicate: HostFilterRow['predicate'],
  values: string[],
  currentTimestamp?: Date
): SplitCondition | null {
  if (!currentTimestamp || values.length < 1) {
    return null;
  }

  const single = predicate === 'is' || predicate === 'is-not';
  const scoped = single ? values.slice(0, 1) : values;
  const beaconVals = scoped.filter((v) => BEACON_STATUS_VALUES.includes(v));
  const hostVals = scoped.filter((v) => HOST_STATUS_VALUES.includes(v));

  const result: SplitCondition = {};
  if (predicate === 'is' || predicate === 'is-in') {
    const beacon = beaconVals.length > 0 ? constructBeaconStatusFilter(beaconVals, currentTimestamp) : null;
    const host = hostVals.length > 0 ? constructHostStatusFilter(hostVals, currentTimestamp) : null;
    if (beacon) {
      result.beacon = beacon;
    }
    if (host) {
      result.host = host;
    }
  } else if (predicate === 'is-not' || predicate === 'is-not-in') {
    const beacon = beaconVals.length > 0 ? constructBeaconStatusFilter(beaconVals, currentTimestamp) : null;
    const host = hostVals.length > 0 ? constructHostStatusFilter(hostVals, currentTimestamp) : null;
    if (beacon) {
      result.beacon = { not: beacon };
    }
    if (host) {
      result.host = { not: host };
    }
  } else {
    return null;
  }

  return (result.beacon || result.host) ? result : null;
}

export function constructTomeFieldsFilterQuery(filter: Filters) {
  const { Tactic, SupportModel } = getTomeFilterNameByTypes(filter.tomeFields);

  if (Tactic.length < 1 && SupportModel.length < 1) {
    return null;
  }

  return {
    "hasTomeWith": {
      ...(Tactic.length && { "tacticIn": Tactic }),
      ...(SupportModel.length && { "supportModelIn": SupportModel })
    }
  };
};

export function constructTomeDefinitionAndValueFilterQuery(filter: Filters) {
  if (!filter.tomeMultiSearch && filter.tomeMultiSearch === "") {
    return null;
  }

  return {
    "or": [
      { "parametersContains": filter.tomeMultiSearch },
      {
        "hasTomeWith": {
          "or": [
            { "paramDefsContains": filter.tomeMultiSearch },
            { "nameContains": filter.tomeMultiSearch },
            { "descriptionContains": filter.tomeMultiSearch }
          ]
        }
      }
    ],
  };
};


export function constructTaskFilterQuery(
  filter: Filters,
  currentTimestamp?: Date,
  questId?: string
) {
  const tomeFieldsFilterQuery = constructTomeFieldsFilterQuery(filter);
  const questParamFilterQuery = constructTomeDefinitionAndValueFilterQuery(filter);
  const beaconFilterQuery = constructBeaconFilterQuery(filter.beaconFields, currentTimestamp);
  if (!questId && !filter.taskOutput && !filter.userId && !beaconFilterQuery && !tomeFieldsFilterQuery && !questParamFilterQuery) {
    return null;
  }

  return {
    "hasTasksWith": {
      ...(filter.taskOutput && { "outputContains": filter.taskOutput }),
      ...(beaconFilterQuery && beaconFilterQuery),
      ...((questId || filter?.userId || tomeFieldsFilterQuery || questParamFilterQuery ) && {
        "hasQuestWith": {
          ...questId && { "id": questId },
          ...filter.userId && { "hasCreatorWith": {"id": filter.userId}},
          ...tomeFieldsFilterQuery && tomeFieldsFilterQuery,
          ...questParamFilterQuery && questParamFilterQuery
        }
      })

    }
  };

};

export function constructQuestFilterQuery(filter: Filters) {
  const tomeFieldsFilterQuery = constructTomeFieldsFilterQuery(filter);
  const questParamFilterQuery = constructTomeDefinitionAndValueFilterQuery(filter);

  if (!filter.questName && !filter.userId && !tomeFieldsFilterQuery && !questParamFilterQuery) {
    return null;
  }

  return {
    ...(filter.questName && { "nameContains": filter.questName }),
    ...filter.userId && { "hasCreatorWith": {"id": filter.userId}},
    ...(tomeFieldsFilterQuery && tomeFieldsFilterQuery),
    ...(questParamFilterQuery && questParamFilterQuery)
  }
}

export function constructHostTaskFilterQuery(
  filter: Filters,
  currentTimestamp?: Date
) {
  const beaconFilterQuery = constructBeaconFilterQuery(filter.beaconFields, currentTimestamp);
  const questFilterQuery = constructQuestFilterQuery(filter);

  if (!filter.taskOutput && !beaconFilterQuery && !questFilterQuery) {
    return null;
  }

  return {
    "hasTasksWith": {
      ...(questFilterQuery && { "hasQuestWith": questFilterQuery }),
      ...(filter.taskOutput && { "outputContains": filter.taskOutput }),
      ...(beaconFilterQuery && beaconFilterQuery),
    }
  };

};

const createRecentlyLostQuery = (currentTimestamp: Date) => ({
  and: [
    { nextSeenAtGTE: sub(currentTimestamp, { minutes: 5 }).toISOString() },
    { nextSeenAtLT: sub(currentTimestamp, { seconds: 30 }).toISOString() }
  ]
});

export function constructBeaconStatusFilter(
  status: Array<string>,
  currentTimestamp?: Date
) {
  if (!currentTimestamp) return null;

  const conditions = [
    ...status.includes(OnlineOfflineFilterType.OnlineBeacons) ? [{ nextSeenAtGTE: sub(currentTimestamp, { seconds: 15 }).toISOString() }] : [],
    ...status.includes(OnlineOfflineFilterType.RecentlyLostBeacons) ? [createRecentlyLostQuery(currentTimestamp)] : [],
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
    ...status.includes(OnlineOfflineFilterType.OfflineHost) ? [{ nextSeenAtLT: sub(currentTimestamp, { seconds: 15 }).toISOString() }] : [],
    ...status.includes(OnlineOfflineFilterType.RecentlyLostHost) ? [createRecentlyLostQuery(currentTimestamp)] : [],
  ]

  if (conditions.length === 0) return null;
  if (conditions.length === 1) return conditions[0];
  return { or: conditions };
}
