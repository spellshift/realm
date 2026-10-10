import { gql } from "@apollo/client";

export const GET_ALL_HOSTS_WITH_TAG_STATS = gql`
  query GetAllHostsWithTagStats(
    $whereOnline: BeaconWhereInput
    $whereRecentlyLost: BeaconWhereInput
  ) {
    hosts {
      edges {
        node {
          id
          lastSeenAt
          tags {
            edges {
              node {
                id
                name
                kind
              }
            }
          }
          onlineBeacons: beacons(where: $whereOnline) {
            totalCount
          }
          recentlyLostBeacons: beacons(where: $whereRecentlyLost) {
            totalCount
          }
          allBeacons: beacons {
            totalCount
          }
          beaconsWithQuests: beacons {
            edges {
              node {
                tasks {
                  edges {
                    node {
                      quest {
                        id
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
`;
