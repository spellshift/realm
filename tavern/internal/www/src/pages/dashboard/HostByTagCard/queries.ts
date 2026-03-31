import { gql } from "@apollo/client";

export const GET_TAGS_FOR_DASHBOARD = gql`
  query GetTagsForDashboard($kind: TagKind!) {
    tags(where: { kind: $kind }) {
      totalCount
      edges {
        node {
          id
          name
          kind
        }
      }
    }
  }
`;

export const GET_HOSTS_BY_TAG = gql`
  query GetHostsByTag(
    $tagName: String!
    $tagKind: TagKind!
    $whereOnline: BeaconWhereInput
    $whereRecentlyLost: BeaconWhereInput
  ) {
    hosts(
      where: {
        hasTagsWith: {
          kind: $tagKind
          nameIn: [$tagName]
        }
      }
    ) {
      totalCount
      edges {
        node {
          id
          name
          primaryIP
          platform
          onlineBeacons: beacons(where: $whereOnline) {
            totalCount
          }
          recentlyLostBeacons: beacons(where: $whereRecentlyLost) {
            totalCount
          }
          allBeacons: beacons {
            totalCount
          }
        }
      }
    }
  }
`;
