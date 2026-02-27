import { gql } from "@apollo/client";

export const GET_SHELL = gql`
  query GetShell($id: ID!) {
    node(id: $id) {
      ... on Shell {
        id
        closedAt
        owner {
          id
          name
        }
        beacon {
          id
          name
          host {
            id
            name
          }
        }
        portals {
          id
          closedAt
        }
      }
    }
  }
`;

export const GET_BEACON_STATUS = gql`
  query GetBeaconStatus($id: ID!) {
    node(id: $id) {
      ... on Beacon {
        lastSeenAt
        nextSeenAt
        interval
      }
    }
  }
`;

export const GET_PORTAL_STATUS = gql`
  query GetPortalStatus($id: ID!) {
    node(id: $id) {
      ... on Portal {
        closedAt
      }
    }
  }
`;

export const GET_SHELL_ACTIVE_USERS = gql`
  query GetShellActiveUsers($id: ID!) {
    node(id: $id) {
      ... on Shell {
        activeUsers {
          edges {
            node {
              id
              name
            }
          }
        }
      }
    }
  }
`;
