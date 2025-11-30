import { HostNode } from "../utils/interfacesQuery";
import Badge from "./tavern-base-ui/badge/Badge";

const HostTile = ({ data }: { data: HostNode }) => {
    return (
        <div className="flex flex-col gap-2">
            <div>{data.name}</div>
            <div className="flex flex-row flex-wrap gap-1">
                <Badge>{data?.primaryIP}</Badge>
                <Badge>{data?.platform}</Badge>
                {data.tags && data?.tags.edges.map((tag) => {
                    return <Badge key={tag.node.id}>{tag.node.name}</Badge>
                })}
            </div>
        </div>
    )
}
export default HostTile;
