import { HostType } from "../utils/consts";
import Badge from "./tavern-base-ui/badge/Badge";

type Props = {
    data: HostType
}
const HostTile = (props: Props) => {
    const { data } = props;
    return (
        <div className="flex flex-col gap-2">
            <div>{data.name}</div>
            <div className="flex flex-row flex-wrap gap-1">
                <Badge>{data?.primaryIP}</Badge>
                <Badge>{data?.platform}</Badge>
                {data.tags && data?.tags.map((tag: any) => {
                    return <Badge key={tag.id}>{tag.name}</Badge>
                })}
            </div>
        </div>
    )
}
export default HostTile;
