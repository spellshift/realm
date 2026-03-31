import { useMemo } from "react";
import { useQuery } from "@apollo/client";
import moment from "moment";
import { GET_HOSTS_BY_TAG } from "./queries";
import {
  GetHostsByTagResponse,
  UseHostByTagDataResult,
  HostBeaconCounts,
} from "./types";
import { testData } from "./testData";

interface UseHostByTagDataParams {
  tagName: string;
  tagKind: "group" | "service";
}

export const useHostByTagData = ({
  tagName,
  tagKind,
}: UseHostByTagDataParams): UseHostByTagDataResult => {
  const queryVariables = useMemo(() => {
    const currentTimestamp = moment();

    const whereOnline = {
      nextSeenAtGTE: currentTimestamp
        .clone()
        .subtract(15, "seconds")
        .toISOString(),
    };

    const whereRecentlyLost = {
      and: [
        {
          nextSeenAtGTE: currentTimestamp
            .clone()
            .subtract(5, "minutes")
            .toISOString(),
        },
        {
          nextSeenAtLT: currentTimestamp
            .clone()
            .subtract(30, "seconds")
            .toISOString(),
        },
      ],
    };

    return {
      tagName,
      tagKind,
      whereOnline,
      whereRecentlyLost,
    };
  }, [tagName, tagKind]);

  const { data, loading, error } = useQuery<GetHostsByTagResponse>(
    GET_HOSTS_BY_TAG,
    {
      variables: queryVariables,
      fetchPolicy: "cache-and-network",
    }
  );

  const hosts = useMemo((): HostBeaconCounts[] => {
    return testData.hosts;
    // if (!data?.hosts?.edges) return [];
    // return data.hosts.edges.map((edge) => edge.node);
  }, [data]);

  const totalOnlineHosts = useMemo((): number => {
    return hosts.filter((host) => host.onlineBeacons.totalCount > 0).length;
  }, [hosts]);

  const totalOnlineBeacons = useMemo((): number => {
    return hosts.reduce((sum, host) => {
      return sum + host.onlineBeacons.totalCount;
    }, 0);
  }, [hosts]);

  return {
    hosts,
    totalOnlineHosts,
    totalOnlineBeacons,
    loading,
    error: error ? new Error(error.message) : undefined,
  };
};
