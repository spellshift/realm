import { gql } from "@apollo/client";

export const GET_SHELL = gql`
  query GetShell($id: ID!) {
    node(id: $id) {
      ... on Shell {
        id
        createdAt
        closedAt
        owner {
          id
          name
        }
        beacon {
          id
          name
          principal
          agentIdentifier
          interval
          transport
          host {
            id
            name
            primaryIP
            externalIP
            platform
            credentials {
              edges {
                node {
                  id
                  principal
                  kind
                  secret
                  createdAt
                }
              }
            }
            tags {
              edges {
                node {
                  id
                  name
                  kind
                }
              }
            }
          }
        }
        portals {
          id
          closedAt
        }
        pivots {
          edges {
            node {
              id
              createdAt
              kind
              destination
              port
              closedAt
            }
          }
        }
      }
    }
  }
`;

export const GET_SHELL_HISTORY = gql`
  query GetShellHistory($id: ID!) {
    node(id: $id) {
      ... on Shell {
        id
        closedAt
        portals {
          id
          closedAt
        }
        pivots {
          edges {
            node {
              id
              createdAt
              kind
              destination
              port
              closedAt
            }
          }
        }
        shellTasks(last: 100) {
          totalCount
          pageInfo {
            hasNextPage
            hasPreviousPage
            startCursor
            endCursor
          }
          edges {
            cursor
            node {
              id
              createdAt
              claimedAt
              execStartedAt
              execFinishedAt
              input
              output
              error
              creator {
                id
                name
                photoURL
              }
              reportedCredentials {
                id
                principal
                kind
              }
              reportedFiles {
                id
                path
                size
              }
              reportedProcesses {
                id
              }
            }
          }
        }
      }
    }
    crossSessionShells: shells(last: 20, where: { hasShellTasks: true }) {
      edges {
        node {
          id
          beacon {
            id
            name
            host {
              id
              name
            }
          }
          shellTasks(last: 40) {
            edges {
              node {
                id
                createdAt
                claimedAt
                execStartedAt
                execFinishedAt
                input
                output
                error
                creator {
                  id
                  name
                  photoURL
                }
                reportedCredentials {
                  id
                  principal
                  kind
                }
                reportedFiles {
                  id
                  path
                  size
                }
                reportedProcesses {
                  id
                }
              }
            }
          }
        }
      }
    }
  }
`;

export const GET_SHELL_KNOWLEDGE_GRAPH = gql`
  query GetShellKnowledgeGraph($hostId: ID!) {
    currentHost: node(id: $hostId) {
      ... on Host {
        id
        name
        primaryIP
        externalIP
        platform
        credentials {
          totalCount
          edges {
            node {
              id
              principal
              kind
              secret
              createdAt
              lastModifiedAt
            }
          }
        }
        files(last: 15) {
          totalCount
          edges {
            node {
              id
              path
              size
              owner
              permissions
              createdAt
            }
          }
        }
        processes {
          totalCount
        }
        beacons {
          totalCount
          edges {
            node {
              id
              name
              principal
              lastSeenAt
              shells {
                edges {
                  node {
                    id
                    createdAt
                    closedAt
                    owner {
                      id
                      name
                    }
                    pivots {
                      edges {
                        node {
                          id
                          createdAt
                          kind
                          destination
                          port
                          closedAt
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
    hosts(first: 50) {
      totalCount
      edges {
        node {
          id
          name
          primaryIP
          externalIP
          platform
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
          beacons {
            totalCount
            edges {
              node {
                id
                name
                principal
                lastSeenAt
                shells {
                  edges {
                    node {
                      id
                      createdAt
                      closedAt
                      owner {
                        id
                        name
                      }
                      pivots {
                        edges {
                          node {
                            id
                            createdAt
                            kind
                            destination
                            port
                            closedAt
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
          }
          credentials {
            totalCount
            edges {
              node {
                id
                principal
                kind
                secret
                createdAt
              }
            }
          }
          files(last: 15) {
            totalCount
            edges {
              node {
                id
                path
                size
                owner
                permissions
                createdAt
              }
            }
          }
        }
      }
    }
    networkHosts(first: 100) {
      totalCount
      edges {
        node {
          id
          name
          hostname
          ipAddress
          subnet
          openPorts
          host {
            id
            name
            primaryIP
            externalIP
            platform
            lastSeenAt
            credentials {
              totalCount
              edges {
                node {
                  id
                  principal
                  kind
                  secret
                  createdAt
                }
              }
            }
          }
          networks {
            edges {
              node {
                id
                name
                subnet
                description
              }
            }
          }
        }
      }
    }
    networks(first: 50) {
      totalCount
      edges {
        node {
          id
          name
          subnet
          description
          networkHosts {
            edges {
              node {
                id
                name
                hostname
                ipAddress
                subnet
                openPorts
              }
            }
          }
        }
      }
    }
  }
`;

export const CREATE_NETWORK_MUTATION = gql`
  mutation CreateNetworkFromShell($input: CreateNetworkInput!) {
    createNetwork(input: $input) {
      id
      name
      subnet
      description
    }
  }
`;

export const UPDATE_NETWORK_MUTATION = gql`
  mutation UpdateNetworkFromShell($networkID: ID!, $input: UpdateNetworkInput!) {
    updateNetwork(networkID: $networkID, input: $input) {
      id
      name
      subnet
      description
    }
  }
`;

export const DELETE_NETWORK_MUTATION = gql`
  mutation DeleteNetworkFromShell($networkID: ID!) {
    deleteNetwork(networkID: $networkID)
  }
`;

export const CREATE_NETWORK_HOST_MUTATION = gql`
  mutation CreateNetworkHostFromShell($input: CreateNetworkHostInput!) {
    createNetworkHost(input: $input) {
      id
      name
      hostname
      ipAddress
      subnet
      openPorts
      host {
        id
        name
      }
      networks {
        edges {
          node {
            id
            name
            subnet
          }
        }
      }
    }
  }
`;

export const UPDATE_NETWORK_HOST_MUTATION = gql`
  mutation UpdateNetworkHostFromShell($networkHostID: ID!, $input: UpdateNetworkHostInput!) {
    updateNetworkHost(networkHostID: $networkHostID, input: $input) {
      id
      name
      hostname
      ipAddress
      subnet
      openPorts
    }
  }
`;

export const DELETE_NETWORK_HOST_MUTATION = gql`
  mutation DeleteNetworkHostFromShell($networkHostID: ID!) {
    deleteNetworkHost(networkHostID: $networkHostID)
  }
`;

export const CREATE_CREDENTIAL_MUTATION = gql`
  mutation CreateHostCredentialFromShell($input: CreateHostCredentialInput!) {
    createCredential(input: $input) {
      id
      principal
      kind
      secret
      createdAt
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

export const CLOSE_PORTAL_MUTATION = gql`
  mutation ClosePortal($id: ID!) {
    closePortal(portalID: $id) {
      id
      closedAt
    }
  }
`;

export const CREATE_SHELL_MUTATION = gql`
  mutation CreateShellFromShellPage($beaconId: ID!) {
    createShell(input: { beaconID: $beaconId }) {
      id
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
              photoURL
            }
          }
        }
      }
    }
  }
`;

export const GET_SHELL_TOMES = gql`
  query GetShellTomes {
    tomes {
      edges {
        node {
          id
          name
          description
          tactic
          eldritch
          paramDefs
          supportModel
        }
      }
    }
  }
`;
