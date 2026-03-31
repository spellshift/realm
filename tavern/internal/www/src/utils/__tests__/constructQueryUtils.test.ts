import { describe, it, expect } from 'vitest';
import {
  constructTagFieldsQuery,
  constructHostFieldQuery,
  constructBeaconFilterQuery,
  constructTomeFieldsFilterQuery,
  constructTomeDefinitionAndValueFilterQuery,
  constructTaskFilterQuery,
  constructQuestFilterQuery,
  constructHostTaskFilterQuery,
  constructBeaconStatusFilter,
  constructHostStatusFilter,
} from '../constructQueryUtils';
import { Filters } from '../../context/FilterContext';
import { FilterBarOption } from '../interfacesUI';
import { OnlineOfflineFilterType } from '../enums';

const emptyFilters: Filters = {
  questName: '',
  taskOutput: '',
  beaconFields: [],
  tomeFields: [],
  tomeMultiSearch: '',
  assetName: '',
  userId: '',
};

const createBeaconField = (kind: string, name: string): FilterBarOption => ({
  kind,
  id: '1',
  name,
});

const FIXED_TIMESTAMP = new Date('2024-01-15T12:00:00.000Z');

describe('constructTagFieldsQuery', () => {
  it('returns null when no groups or services', () => {
    expect(constructTagFieldsQuery([], [])).toBeNull();
  });

  it('returns tag queries for groups and services', () => {
    const result = constructTagFieldsQuery(['admin'], ['web']);
    expect(result).toEqual([
      { hasTagsWith: { kind: 'group', nameIn: ['admin'] } },
      { hasTagsWith: { kind: 'service', nameIn: ['web'] } },
    ]);
  });
});

describe('constructHostFieldQuery', () => {
  it('returns null when no filters provided', () => {
    expect(constructHostFieldQuery([], [], [], [], [], [])).toBeNull();
  });

  it.each([
    { args: [[], [], [], ['host1'], [], []], expected: { nameIn: ['host1'] }, desc: 'hosts' },
    { args: [[], [], ['linux'], [], [], []], expected: { platformIn: ['linux'] }, desc: 'platforms' },
    { args: [[], [], [], [], ['192.168.1.1'], []], expected: { primaryIPIn: ['192.168.1.1'] }, desc: 'primaryIP' },
  ])('includes $desc in hasHostWith', ({ args, expected }) => {
    const result = constructHostFieldQuery(...(args as Parameters<typeof constructHostFieldQuery>));
    expect(result).toEqual({ hasHostWith: expected });
  });

  it('combines multiple filters', () => {
    const result = constructHostFieldQuery(['admin'], ['web'], ['linux'], ['server1'], [], []);
    expect(result?.hasHostWith).toMatchObject({
      nameIn: ['server1'],
      platformIn: ['linux'],
    });
    expect(result?.hasHostWith?.and).toBeDefined();
  });
});

describe('constructBeaconFilterQuery', () => {
  it('returns null when no beacon fields', () => {
    expect(constructBeaconFilterQuery([])).toBeNull();
  });

  it.each([
    { kind: 'beacon', name: 'beacon1', expectedKey: 'nameIn' },
    { kind: 'principal', name: 'root', expectedKey: 'principalIn' },
    { kind: 'transport', name: 'http', expectedKey: 'transportIn' },
  ])('maps $kind field to $expectedKey', ({ kind, name, expectedKey }) => {
    const result = constructBeaconFilterQuery([createBeaconField(kind, name)]);
    expect(result?.hasBeaconWith?.[expectedKey]).toEqual([name]);
  });

  it('nests host filters under hasHostWith', () => {
    const result = constructBeaconFilterQuery([createBeaconField('host', 'server1')]);
    expect(result?.hasBeaconWith?.hasHostWith?.nameIn).toEqual(['server1']);
  });
});

describe('constructTomeFieldsFilterQuery', () => {
  it('returns null when no tome fields', () => {
    expect(constructTomeFieldsFilterQuery(emptyFilters)).toBeNull();
  });

  it.each([
    { kind: 'Tactic', value: 'persistence', expectedKey: 'tacticIn' },
    { kind: 'SupportModel', value: 'FIRST_PARTY', expectedKey: 'supportModelIn' },
  ])('maps $kind to $expectedKey', ({ kind, value, expectedKey }) => {
    const filters: Filters = {
      ...emptyFilters,
      tomeFields: [{ kind, id: '1', name: value, value }],
    };
    const result = constructTomeFieldsFilterQuery(filters);
    const hasTomeWith = result?.hasTomeWith as Record<string, string[]> | undefined;
    expect(hasTomeWith?.[expectedKey]).toEqual([value]);
  });
});

describe('constructTomeDefinitionAndValueFilterQuery', () => {
  it('returns null when tomeMultiSearch is empty', () => {
    expect(constructTomeDefinitionAndValueFilterQuery(emptyFilters)).toBeNull();
  });

  it('searches across parameters, paramDefs, name, and description', () => {
    const result = constructTomeDefinitionAndValueFilterQuery({
      ...emptyFilters,
      tomeMultiSearch: 'password',
    });
    expect(result?.or).toHaveLength(2);
    expect(result?.or?.[0]).toEqual({ parametersContains: 'password' });
    expect(result?.or?.[1]?.hasTomeWith?.or).toHaveLength(3);
  });
});

describe('constructTaskFilterQuery', () => {
  it('returns null when no filters are set', () => {
    expect(constructTaskFilterQuery(emptyFilters)).toBeNull();
  });

  it('includes taskOutput filter', () => {
    const result = constructTaskFilterQuery({ ...emptyFilters, taskOutput: 'error' });
    expect(result?.hasTasksWith?.outputContains).toBe('error');
  });

  it('includes questId in hasQuestWith', () => {
    const result = constructTaskFilterQuery(emptyFilters, undefined, 'quest-123');
    expect(result?.hasTasksWith?.hasQuestWith?.id).toBe('quest-123');
  });

  it('includes hasCreatorWith when userId is set', () => {
    const result = constructTaskFilterQuery({ ...emptyFilters, userId: 'user-456' });
    expect(result?.hasTasksWith?.hasQuestWith?.hasCreatorWith).toEqual({ id: 'user-456' });
  });

  it('combines userId with questId and taskOutput', () => {
    const result = constructTaskFilterQuery(
      { ...emptyFilters, userId: 'user-456', taskOutput: 'success' },
      undefined,
      'quest-789'
    );
    expect(result?.hasTasksWith).toMatchObject({
      outputContains: 'success',
      hasQuestWith: {
        id: 'quest-789',
        hasCreatorWith: { id: 'user-456' },
      },
    });
  });
});

describe('constructQuestFilterQuery', () => {
  it('returns null when no filters are set', () => {
    expect(constructQuestFilterQuery(emptyFilters)).toBeNull();
  });

  it('includes questName filter', () => {
    const result = constructQuestFilterQuery({ ...emptyFilters, questName: 'my-quest' });
    expect(result?.nameContains).toBe('my-quest');
  });

  it('includes hasCreatorWith when userId is set', () => {
    const result = constructQuestFilterQuery({ ...emptyFilters, userId: 'user-123' });
    expect(result?.hasCreatorWith).toEqual({ id: 'user-123' });
  });

  it('combines userId with other filters', () => {
    const result = constructQuestFilterQuery({
      ...emptyFilters,
      questName: 'quest',
      userId: 'user-123',
      tomeMultiSearch: 'search-term',
    });
    expect(result).toMatchObject({
      nameContains: 'quest',
      hasCreatorWith: { id: 'user-123' },
    });
    expect(result?.or).toBeDefined(); 
  });
});

describe('constructHostTaskFilterQuery', () => {
  it('returns null when no filters are set', () => {
    expect(constructHostTaskFilterQuery(emptyFilters)).toBeNull();
  });

  it('includes taskOutput filter', () => {
    const result = constructHostTaskFilterQuery({ ...emptyFilters, taskOutput: 'completed' });
    expect(result?.hasTasksWith?.outputContains).toBe('completed');
  });

  it('includes hasCreatorWith when userId is set', () => {
    const result = constructHostTaskFilterQuery({ ...emptyFilters, userId: 'user-789' });
    expect(result?.hasTasksWith?.hasQuestWith?.hasCreatorWith).toEqual({ id: 'user-789' });
  });
});

describe.each([
  {
    name: 'constructBeaconStatusFilter',
    fn: constructBeaconStatusFilter,
    singleStatus: OnlineOfflineFilterType.OnlineBeacons,
    recentlyLost: OnlineOfflineFilterType.RecentlyLostBeacons,
    singleExpectedKey: 'nextSeenAtGTE',
  },
  {
    name: 'constructHostStatusFilter',
    fn: constructHostStatusFilter,
    singleStatus: OnlineOfflineFilterType.OfflineHost,
    recentlyLost: OnlineOfflineFilterType.RecentlyLostHost,
    singleExpectedKey: 'nextSeenAtLT',
  },
])('$name', ({ fn, singleStatus, recentlyLost, singleExpectedKey }) => {
  it('returns null when no timestamp provided', () => {
    expect(fn([singleStatus])).toBeNull();
  });

  it('returns null when status array is empty', () => {
    expect(fn([], FIXED_TIMESTAMP)).toBeNull();
  });

  it('returns single condition for one status', () => {
    const result = fn([singleStatus], FIXED_TIMESTAMP);
    expect(result).toHaveProperty(singleExpectedKey);
  });

  it('returns and-condition for recently lost status', () => {
    const result = fn([recentlyLost], FIXED_TIMESTAMP);
    expect(result).toHaveProperty('and');
    expect((result as any).and).toHaveLength(2);
  });

  it('returns or-condition when multiple statuses', () => {
    const result = fn([singleStatus, recentlyLost], FIXED_TIMESTAMP);
    expect(result).toHaveProperty('or');
    expect((result as any).or).toHaveLength(2);
  });
});
