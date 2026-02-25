import { gql } from "@apollo/client";
import { OnlineOfflineFilterType } from "../../utils/enums";
import { FilterBarOption } from "../../utils/interfacesUI";

export const GET_BEACON_FILTER_OPTIONS = gql`
    query GetBeaconFilterOptions($groupTag: TagWhereInput, $serviceTag: TagWhereInput) {
        groupTags: tags(where: $groupTag) {
            edges {
                node {
                    id
                    name
                    kind
                }
            }
        }
        serviceTags: tags(where: $serviceTag) {
            edges {
                node {
                    id
                    name
                    kind
                }
            }
        }
        beacons {
            edges {
                node {
                    id
                    name
                    principal
                }
            }
        }
        hosts {
            edges {
                node {
                    id
                    name
                    primaryIP
                }
            }
        }
    }
`;

export const ONLINE_OFFLINE_OPTIONS: FilterBarOption[] = [
    {
        id: OnlineOfflineFilterType.OnlineBeacons,
        name: "Online beacons",
        value: OnlineOfflineFilterType.OnlineBeacons,
        label: "Online beacons",
        kind: "onlineOfflineStatus"
    },
    {
        id: OnlineOfflineFilterType.OfflineHost,
        name: "Offline hosts",
        value: OnlineOfflineFilterType.OfflineHost,
        label: "Offline hosts",
        kind: "onlineOfflineStatus"
    },
    {
        id: OnlineOfflineFilterType.RecentlyLostHost,
        name: "Recently lost host",
        value: OnlineOfflineFilterType.RecentlyLostHost,
        label: "Recently lost host",
        kind: "onlineOfflineStatus"
    },
    {
        id: OnlineOfflineFilterType.RecentlyLostBeacons,
        name: "Recently lost beacons",
        value: OnlineOfflineFilterType.RecentlyLostBeacons,
        label: "Recently lost beacons",
        kind: "onlineOfflineStatus"
    },
];
