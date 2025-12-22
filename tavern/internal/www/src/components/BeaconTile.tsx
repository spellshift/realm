import { BeaconNode, TagEdge } from "../utils/interfacesQuery";
import { checkIfBeaconOffline } from "../utils/utils";
import Badge from "./tavern-base-ui/badge/Badge";
import { Globe, Network } from "lucide-react";

type Props = {
    beaconData: BeaconNode
}
const BeaconTile = (props: Props) => {
    const { beaconData } = props;
    const beaconOffline = checkIfBeaconOffline(beaconData);
    return (
        <div className="flex flex-col gap-1">
            <div className="flex flex-row gap-4">{beaconData.name}</div>
            <div className="flex flex-row flex-wrap gap-1">
                {(beaconData.principal && beaconData.principal !== "") &&
                    <Badge>{beaconData.principal}</Badge>
                }
                <Badge>{beaconData?.host?.name}</Badge>
                {beaconData?.host?.primaryIP && (
                    <Badge leftIcon={<Network className="h-3 w-3" />}>{beaconData?.host?.primaryIP}</Badge>
                )}
                {beaconData?.host?.externalIP && (
                    <Badge leftIcon={<Globe className="h-3 w-3" />}>{beaconData?.host?.externalIP}</Badge>
                )}
                <Badge>{beaconData?.host?.platform}</Badge>
                {beaconData?.host?.tags && beaconData?.host?.tags?.edges.map((tag: TagEdge) => {
                    return <Badge key={tag.node.id}>{tag.node.name}</Badge>
                })}
                {beaconOffline && <Badge>Offline</Badge>}
            </div>
        </div>
    );
}
export default BeaconTile;
