import { describe, it, expect, beforeEach, vi } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { ApolloClient, ApolloLink, ApolloProvider, InMemoryCache, Observable } from '@apollo/client';
import React from 'react';
import { useOnlineBeaconIds } from '../useOnlineBeaconIds';
import { HostFilterRow } from '../../../../utils/interfacesUI';

const beaconData = {
  beacons: {
    totalCount: 1,
    pageInfo: {
      hasNextPage: false,
      hasPreviousPage: false,
      startCursor: 'start',
      endCursor: 'end',
    },
    edges: [
      {
        node: {
          id: 'beacon-1',
          principal: 'root',
          transport: 'TRANSPORT_GRPC',
          host: { id: 'host-1' },
        },
      },
    ],
  },
};

function setup(captured: unknown[]) {
  const link = new ApolloLink((operation) => {
    captured.push(operation.variables);
    return new Observable((observer) => {
      observer.next({ data: beaconData });
      observer.complete();
    });
  });
  const client = new ApolloClient({ link, cache: new InMemoryCache() });

  return function Wrapper({ children }: { children: React.ReactNode }) {
    return <ApolloProvider client={client}>{children}</ApolloProvider>;
  };
}

describe('useOnlineBeaconIds with filter rows', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('merges row filters into the beacon query while keeping the online filter', async () => {
    const captured: Array<Record<string, any>> = [];
    const props = {
      rows: [
        { id: 'row-1', field: 'platform', predicate: 'is', values: ['PLATFORM_LINUX'] },
        { id: 'row-2', field: 'principal', predicate: 'contains', values: ['ro'] },
      ] as HostFilterRow[],
    };
    const { result } = renderHook(() => useOnlineBeaconIds(props), {
      wrapper: setup(captured),
    });

    await waitFor(() => {
      expect(result.current.beaconIds).toEqual(['beacon-1']);
    });

    expect(captured).toHaveLength(1);
    expect(captured[0].where).toMatchObject({
      hasHostWith: { platformIn: ['PLATFORM_LINUX'] },
      principalContains: 'ro',
    });
    expect(typeof captured[0].where.nextSeenAtGTE).toBe('string');
    expect(captured[0].orderBy).toEqual([{ field: 'CREATED_AT', direction: 'DESC' }]);
  });

  it('ignores incomplete rows', async () => {
    const captured: Array<Record<string, any>> = [];
    const props = {
      rows: [{ id: 'row-1', field: 'platform', predicate: 'is', values: [] }] as HostFilterRow[],
    };
    const { result } = renderHook(() => useOnlineBeaconIds(props), {
      wrapper: setup(captured),
    });

    await waitFor(() => {
      expect(result.current.beaconIds).toEqual(['beacon-1']);
    });

    expect(captured).toHaveLength(1);
    expect(Object.keys(captured[0].where)).toEqual(['nextSeenAtGTE']);
  });

  it('adds an id filter when viewing only selected beacons', async () => {
    const captured: Array<Record<string, any>> = [];
    const props = { rows: [] as HostFilterRow[], selectedBeaconIds: ['beacon-9'], viewOnlySelected: true };
    const { result } = renderHook(() => useOnlineBeaconIds(props), {
      wrapper: setup(captured),
    });

    await waitFor(() => {
      expect(result.current.beaconIds).toEqual(['beacon-1']);
    });

    expect(captured[0].where).toMatchObject({ idIn: ['beacon-9'] });
  });
});
