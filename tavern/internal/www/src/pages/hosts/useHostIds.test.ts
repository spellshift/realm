import { describe, it, expect } from 'vitest';
import { getHostIdsQuery } from './useHostIds';
import { defaultSorts } from '../../context/SortContext/sortingUtils';
import { PageNavItem } from '../../utils/enums';
import { Filters } from '../../context/FilterContext/FilterContext';

const defaultFilters: Filters = {
  questName: '',
  taskOutput: '',
  beaconFields: [],
  tomeFields: [],
  tomeMultiSearch: '',
  assetName: '',
  userId: '',
  agentOnly: false,
};

const hostSort = defaultSorts[PageNavItem.hosts];

describe('getHostIdsQuery tag-kind exclusion', () => {
  it('excludes hosts that already have a tag of the given kind', () => {
    const variables = getHostIdsQuery(defaultFilters, undefined, hostSort, 100, 'group');

    expect(variables.where).toMatchObject({
      not: { hasTagsWith: { kind: 'group' } },
    });
  });

  it('excludes service-tagged hosts when adding a service tag', () => {
    const variables = getHostIdsQuery(defaultFilters, undefined, hostSort, 100, 'service');

    expect(variables.where).toMatchObject({
      not: { hasTagsWith: { kind: 'service' } },
    });
  });

  it('does not add an exclusion filter when no kind is given', () => {
    const variables = getHostIdsQuery(defaultFilters, undefined, hostSort);

    expect(variables.where).not.toHaveProperty('not');
  });
});
