import { useEffect, useMemo, useRef, useState } from "react";
import { NetworkStatus, useQuery } from "@apollo/client";
import moment from "moment";
import { GET_ALL_HOSTS_WITH_TAG_STATS } from "./queries";
import { GetAllHostsWithTagStatsResponse, TagKind, TagRow } from "./types";

const computeQueryVariables = () => {
  const now = moment();
  return {
    whereOnline: {
      nextSeenAtGTE: now.clone().subtract(15, "seconds").toISOString(),
    },
    whereRecentlyLost: {
      and: [
        { nextSeenAtGTE: now.clone().subtract(5, "minutes").toISOString() },
        { nextSeenAtLT: now.clone().subtract(30, "seconds").toISOString() },
      ],
    },
  };
};

export const useTagSummaryData = (tagKind: TagKind): {
  rows: TagRow[];
  loading: boolean;
  error?: Error;
} => {
  const [queryVariables, setQueryVariables] = useState(computeQueryVariables);

  useEffect(() => {
    const id = setInterval(() => setQueryVariables(computeQueryVariables()), 30000);
    return () => clearInterval(id);
  }, []);

  const { data, networkStatus, error } = useQuery<GetAllHostsWithTagStatsResponse>(
    GET_ALL_HOSTS_WITH_TAG_STATS,
    {
      variables: queryVariables,
      fetchPolicy: "cache-and-network",
      notifyOnNetworkStatusChange: true,
    }
  );

  const previousRowsRef = useRef<TagRow[]>([]);

  const rows = useMemo((): TagRow[] => {
    if (!data?.hosts?.edges) return previousRowsRef.current;

    const tagMap = new Map<string, TagRow>();
    const tagQuestIds = new Map<string, Set<string>>();

    for (const { node: host } of data.hosts.edges) {
      const matchingTags = host.tags.edges
        .map((e) => e.node)
        .filter((tag) => tag.kind === tagKind);

      const hostQuestIds = host.beaconsWithQuests.edges.flatMap((b) =>
        b.node.tasks.edges.map((t) => t.node.quest.id)
      );

      for (const tag of matchingTags) {
        const existing = tagMap.get(tag.name) ?? {
          tagId: tag.id,
          tagName: tag.name,
          tagKind,
          onlineHosts: 0,
          lostHosts: 0,
          onlineBeacons: 0,
          recentlyLostBeacons: 0,
          lastCallbackAt: null,
          questCount: 0,
        };

        if (host.onlineBeacons.totalCount > 0) {
          existing.onlineHosts += 1;
        } else {
          existing.lostHosts += 1;
        }

        existing.onlineBeacons += host.onlineBeacons.totalCount;
        existing.recentlyLostBeacons += host.recentlyLostBeacons.totalCount;

        if (host.lastSeenAt) {
          if (
            !existing.lastCallbackAt ||
            new Date(host.lastSeenAt) > new Date(existing.lastCallbackAt)
          ) {
            existing.lastCallbackAt = host.lastSeenAt;
          }
        }

        const questSet = tagQuestIds.get(tag.name) ?? new Set<string>();
        for (const id of hostQuestIds) questSet.add(id);
        tagQuestIds.set(tag.name, questSet);

        tagMap.set(tag.name, existing);
      }
    }

    const computed = Array.from(tagMap.values()).map((row) => ({
      ...row,
      questCount: tagQuestIds.get(row.tagName)?.size ?? 0,
    })).sort((a, b) =>
      a.tagName.localeCompare(b.tagName)
    );

    previousRowsRef.current = computed;
    return computed;
  }, [data, tagKind]);

  return {
    rows,
    loading: networkStatus === NetworkStatus.loading,
    error: error ? new Error(error.message) : undefined,
  };
};
