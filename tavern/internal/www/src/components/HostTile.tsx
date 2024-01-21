import { Badge } from "@chakra-ui/react";
import { HostType } from "../utils/consts";

type Props = {
    data: HostType
}
const HostTile = (props: Props) => {
    const { data } = props;
    return (
        <div className="flex flex-col gap-2">
            <div>{data.name}</div>
            <div className="flex flex-row flex-wrap gap-1">
                {data.tags && data?.tags.map((tag: any) => {
                    return <Badge key={tag.id}>{tag.name}</Badge>
                })}
                <Badge>{data?.primaryIP}</Badge>
                <Badge>{data?.platform}</Badge>
            </div>
        </div>
    )
}
export default HostTile;
