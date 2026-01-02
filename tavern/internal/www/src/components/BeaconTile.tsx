import { BugAntIcon } from "@heroicons/react/24/outline";
import { BeaconNode, TagEdge } from "../utils/interfacesQuery";
import { checkIfBeaconOffline } from "../utils/utils";
import Badge from "./tavern-base-ui/badge/Badge";
import { Globe, Network } from "lucide-react";


type Props = {
    beacon: BeaconNode;
    isBeaconIconVisible?: boolean;
}
const BeaconTile = (props: Props) => {
    const { beacon, isBeaconIconVisible } = props;
    const {
        host,
        principal,
        name,
        transport
    } = beacon;
    const beaconOffline = checkIfBeaconOffline(beacon);

    return (
        <div className=" flex flex-row gap-4">
            {isBeaconIconVisible && <BugAntIcon className="h-5 w-5 mt-2" />}
            <div className="flex flex-col gap-1 ">
                <div className="text-gray-600">
                    {name}@{host?.name}
                </div>
                <div className="flex flex-row gap-2 flex-wrap">
                    {(principal && principal !== "") &&
                        <Badge>{principal}</Badge>
                    }
                    {transport &&
                        <Badge>{transport}</Badge>
                    }
                    {host?.primaryIP && (
                        <Badge leftIcon={<Network className="h-3 w-3" />}>{host?.primaryIP}</Badge>
                    )}
                    {host?.externalIP && (
                        <Badge leftIcon={<Globe className="h-3 w-3" />}>{host?.externalIP}</Badge>
                    )}
                    {host?.platform &&
                        <Badge>{host?.platform}</Badge>
                    }
                    {host?.tags && host?.tags?.edges?.map((tag: TagEdge) => {
                        return <Badge key={tag.node.id}>{tag.node.name}</Badge>
                    })}
                    {beaconOffline && <Badge>Offline</Badge>}
                </div>
            </div>
        </div>
    );
};
export default BeaconTile;
