import { SupportedPlatforms } from "../utils/enums";
import { HostNode } from "../utils/interfacesQuery";
import { getEnumKey } from "../utils/utils";
import Badge from "./tavern-base-ui/badge/Badge";
import { Globe, Network } from "lucide-react";

const HostTile = ({ data }: { data: HostNode }) => {
    return (
        <div className="flex flex-col gap-2">
            <div>{data.name}</div>
            <div className="flex flex-row flex-wrap gap-1">
                {data?.primaryIP && (
                    <Badge leftIcon={<Network className="h-3 w-3" />}>{data?.primaryIP}</Badge>
                )}
                {data?.externalIP && (
                    <Badge leftIcon={<Globe className="h-3 w-3" />}>{data?.externalIP}</Badge>
                )}
                {data?.platform && (<Badge>{getEnumKey(SupportedPlatforms, data?.platform)}</Badge>)}
                {data.tags && data?.tags.edges.map((tag) => {
                    return <Badge key={tag.node.id}>{tag.node.name}</Badge>
                })}
            </div>
        </div>
    )
}
export default HostTile;
