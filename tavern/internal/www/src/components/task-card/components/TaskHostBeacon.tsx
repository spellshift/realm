import { BugAntIcon } from "@heroicons/react/24/outline";
import Badge from "../../tavern-base-ui/badge/Badge";
import { FC } from "react";
import { checkIfBeaconOffline } from "../../../utils/utils";
import { BeaconNode, TagEdge } from "../../../utils/interfacesQuery";

interface TaskHostBeaconType {
    beacon: BeaconNode
}

const TaskHostBeacon: FC<TaskHostBeaconType> = ({ beacon }) => {
    const {
        host,
        principal,
        name
    } = beacon;
    const beaconOffline = checkIfBeaconOffline(beacon);

    return (
        <div className=" flex flex-row gap-4">
            <BugAntIcon className="h-5 w-5 mt-2" />
            <div className="flex flex-col gap-1 ">
                <div className="text-gray-600">
                    {name}@{host?.name}
                </div>
                <div className="flex flex-row gap-2 flex-wrap">
                    {(principal && principal !== "") &&
                        <Badge>{principal}</Badge>
                    }
                    {(host?.primaryIP && host?.primaryIP !== "") &&
                        <Badge>{host?.primaryIP}</Badge>
                    }
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
export default TaskHostBeacon;
