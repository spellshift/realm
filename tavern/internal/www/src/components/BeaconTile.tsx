import { checkIfBeaconOffline } from "../utils/utils";
import Badge from "./tavern-base-ui/badge/Badge";

type Props = {
    beaconData: {
        name: string;
        lastSeenAt: string;
        interval: number;
        principal?: string;
        host: {
            id: string;
            tags?: Array<any>;
            name: string;
            primaryIP?: string;
            platform?: string;
        }
    }
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
                <Badge>{beaconData?.host?.primaryIP}</Badge>
                <Badge>{beaconData?.host?.platform}</Badge>
                {beaconData?.host?.tags && beaconData?.host?.tags.map((tag: any) => {
                    return <Badge key={tag.id}>{tag.name}</Badge>
                })}
                {beaconOffline && <Badge>Offline</Badge>}
            </div>
        </div>
    );
}
export default BeaconTile;
