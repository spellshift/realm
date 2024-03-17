import { FC } from "react";
import { HostType, TomeTag } from "../../../utils/consts";
import { getOfflineOnlineStatus } from "../../../utils/utils";
import { TaskChartKeys } from "../../../utils/enums";
import Button from "../../../components/tavern-base-ui/button/Button";
import { useNavigate } from "react-router-dom";

type TargetReccomendationProps = {
    hosts: Array<HostType>;
    data: Array<any>;
    tagKind: string;
}

const TargetReccomendation: FC<TargetReccomendationProps> = ({ hosts, data, tagKind }) => {
    const navigation = useNavigate();

    const tagWithFewestTasks = data.length > 0 ? data.find((task: any) => task.name !== "undefined") : null;

    const getTotalActiveBeaconsForTagKind = () => {
        const returnedValue = hosts.reduce((acc, curr) => {
            const matchesGroup = curr?.tags?.find((tag: TomeTag) => { return tag.name === tagWithFewestTasks.name });
            const beaconStatus = getOfflineOnlineStatus(curr.beacons || [])
            if (matchesGroup) {
                return acc += beaconStatus.online;
            }
            return acc;
        }, 0);
        return returnedValue;
    };

    const handleClickQuestDetails = (item: any) => {
        navigation("/quests", {
            state: [{
                'label': item?.name,
                'kind': tagKind,
                'name': item?.name,
                'value': item?.id
            }]
        })
    }

    if (!tagWithFewestTasks) {
        return null;
    }

    return (
        <div className='flex flex-col border-l-4 border-purple-900 px-4 py-2 rounded'>
            <h4 className="font-semibold text-gray-900">Consider targeting the {tagKind} with fewest tasks</h4>
            <p className='text-sm'>{tagWithFewestTasks.name} has {tagWithFewestTasks[TaskChartKeys.taskNoError]} task run and {getTotalActiveBeaconsForTagKind()} online beacons</p>
            <div className='flex flex-row gap-4 mt-2'>
                <Button
                    buttonStyle={{ color: "purple", size: "xs", vPadding: "none", xPadding: "none" }}
                    buttonVariant='ghost'
                    className='hover:underline hover:bg-'
                    onClick={() => {
                        handleClickQuestDetails(tagWithFewestTasks)
                    }}>
                    See quest details
                </Button>
            </div>
        </div>
    );
}
export default TargetReccomendation;
