import { Badge } from "@chakra-ui/react";

type Props = {
    beaconData: {
        name: string;
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
    return (
        <div className="flex flex-col gap-1">
            <div>{beaconData.name}</div>
            <div className="flex flex-row flex-wrap gap-1">
                {beaconData?.host?.tags.map((tag: any)=> {
                    return <Badge key={tag.id}>{tag.name}</Badge>
                })}
                <Badge>{beaconData?.host?.name}</Badge>
                <Badge>{beaconData?.host?.primaryIP}</Badge>
                <Badge>{beaconData?.host?.platform}</Badge>
            </div>
        </div>
    );
}
export default BeaconTile;