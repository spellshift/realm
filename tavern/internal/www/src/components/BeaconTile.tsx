import { Badge } from "@chakra-ui/react";
import { checkIfBeaconOnline } from "../utils/utils";

type Props = {
    beaconData: {
        name: string;
        lastSeenAt: string;
        interval: number;
        host: {
            tags: Array<any>;
            name: string;
            primaryIP: string;
            platform: string;
        }
    }
}
const BeaconTile = (props: Props) => {
    const {beaconData} = props;
    const beaconOffline = checkIfBeaconOnline(beaconData);
    return (
        <div className="flex flex-col gap-1">
            <div className="flex flex-row gap-4">{beaconData.name}</div>
            <div className="flex flex-row flex-wrap gap-1">
                {beaconData?.host?.tags.map((tag: any)=> {
                    return <Badge key={tag.id}>{tag.name}</Badge>
                })}
                <Badge>{beaconData?.host?.name}</Badge>
                <Badge>{beaconData?.host?.primaryIP}</Badge>
                <Badge>{beaconData?.host?.platform}</Badge>
                {beaconOffline && <Badge>Offline</Badge>}
            </div>
        </div>
    );
}
export default BeaconTile;