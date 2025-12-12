import { BugAntIcon } from "@heroicons/react/24/outline";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import { FC } from "react";
import { BeaconType } from "../../../utils/consts";
import { checkIfBeaconOffline } from "../../../utils/utils";

interface TaskHostBeaconType {
    beaconData: BeaconType
}

const TaskHostBeacon: FC<TaskHostBeaconType> = ({ beaconData }) => {
    const {
        host,
        principal,
        name
    } = beaconData;
    const beaconOffline = checkIfBeaconOffline(beaconData);

    return (
        <div className=" flex flex-row gap-4">
            <BugAntIcon className="h-5 w-5 mt-2" />
            <div className="flex flex-col gap-1 ">
                <div className="text-gray-600">
                    {name}@{host.name}
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
                    {host?.tags && host?.tags.map((tag: any) => {
                        return <Badge key={tag.id}>{tag.name}</Badge>
                    })}
                    {beaconOffline && <Badge>Offline</Badge>}
                </div>
            </div>
        </div>
    );
};
export default TaskHostBeacon;
