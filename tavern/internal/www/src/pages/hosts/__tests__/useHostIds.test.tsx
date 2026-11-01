import { describe, it, expect, beforeEach, vi } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { MockedProvider } from '@apollo/client/testing';
import { MemoryRouter } from 'react-router-dom';
import React from 'react';
import { FilterProvider, useFilters } from '../../../context/FilterContext';
import { SortsProvider } from '../../../context/SortContext';
import { useHostIds } from '../useHostIds';
import { GET_HOST_IDS_QUERY } from '../queries';
import { HostOrderField, OrderDirection, TableRowLimit } from '../../../utils/enums';
import { HostFilterRow } from '../../../utils/interfacesUI';

const ORDER_BY = [{ direction: OrderDirection.Desc, field: HostOrderField.CreatedAt }];

const pageInfo = {
  hasNextPage: false,
  hasPreviousPage: false,
  startCursor: 'start',
  endCursor: 'end',
};

const allHostsData = {
  hosts: {
    totalCount: 2,
    pageInfo,
    edges: [{ node: { id: 'host-1' } }, { node: { id: 'host-2' } }],
  },
};

const linuxHostsData = {
  hosts: {
    totalCount: 1,
    pageInfo,
    edges: [{ node: { id: 'host-2' } }],
  },
};

const mocks = [
  {
    request: {
      query: GET_HOST_IDS_QUERY,
      variables: { where: {}, orderBy: ORDER_BY, first: TableRowLimit.HostRowLimit, after: undefined },
    },
    result: { data: allHostsData },
  },
  {
    request: {
      query: GET_HOST_IDS_QUERY,
      variables: {
        where: { platformIn: ['PLATFORM_LINUX'] },
        orderBy: ORDER_BY,
        first: TableRowLimit.HostRowLimit,
        after: undefined,
      },
    },
    result: { data: linuxHostsData },
  },
];

function Wrapper({ children }: { children: React.ReactNode }) {
  return (
    <MemoryRouter>
      <FilterProvider>
        <SortsProvider>
          <MockedProvider mocks={mocks} addTypename={false}>
            {children}
          </MockedProvider>
        </SortsProvider>
      </FilterProvider>
    </MemoryRouter>
  );
}

function useHostIdsWithFilters() {
  const ids = useHostIds();
  const { updateFilters } = useFilters();
  return { ...ids, updateFilters };
}

describe('useHostIds with host filter rows', () => {
  beforeEach(() => {
    sessionStorage.clear();
    vi.clearAllMocks();
  });

  it('keeps showing all hosts while a row is incomplete', async () => {
    const { result } = renderHook(() => useHostIdsWithFilters(), { wrapper: Wrapper });

    await waitFor(() => {
      expect(result.current.hostIds).toEqual(['host-1', 'host-2']);
    });

    const incompleteRow: HostFilterRow = { id: 'row-1', field: 'platform', predicate: 'is', values: [] };
    act(() => {
      result.current.updateFilters({ hostFilters: [incompleteRow] });
    });

    // Give any stray refetch a chance to land; the table must not change.
    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 100));
    });

    expect(result.current.hostIds).toEqual(['host-1', 'host-2']);
    expect(result.current.error).toBeUndefined();
  });

  it('applies the filter once the row is complete', async () => {
    const { result } = renderHook(() => useHostIdsWithFilters(), { wrapper: Wrapper });

    await waitFor(() => {
      expect(result.current.hostIds).toEqual(['host-1', 'host-2']);
    });

    const completeRow: HostFilterRow = { id: 'row-1', field: 'platform', predicate: 'is', values: ['PLATFORM_LINUX'] };
    act(() => {
      result.current.updateFilters({ hostFilters: [completeRow] });
    });

    await waitFor(() => {
      expect(result.current.hostIds).toEqual(['host-2']);
    });
  });
});
