import { RepeatClockIcon, TimeIcon } from "@chakra-ui/icons";
import { Badge } from "@chakra-ui/react";
import { CheckCircleIcon, ExclamationCircleIcon } from "@heroicons/react/24/outline";

type Props = {
    task: any;
}
const TaskStatusBadge = (props: Props) => {
    const { task } = props;

    if (task.error.length > 0) {
        return (
            <div>
                <Badge fontSize='0.8em' size="large" colorScheme="red" variant="solid">
                    <div className="flex flex-row gap-1 justify-center items-center p-1" >
                        <ExclamationCircleIcon className="w-5" color="white" />
                        <div>Error</div>
                    </div>
                </Badge>
            </div>
        )
    }

    if (task.execFinishedAt) {
        return (
            <div>
                <Badge fontSize='0.8em' size="large" colorScheme="green" variant="solid">
                    <div className="flex flex-row gap-1 justify-center items-center p-1" >
                        <CheckCircleIcon className="w-5" color="white" />
                        <div>Finished</div>
                    </div>
                </Badge>
            </div>
        );
    }

    if (task.execStartedAt) {
        return (
            <div>
                <Badge fontSize='0.8em' size="large" colorScheme="gray" variant="outline">
                    <div className="flex flex-row gap-1 justify-center items-center p-1" >
                        <RepeatClockIcon w={4} h={4} color="gray" />
                        <div>In-Progress</div>
                    </div>
                </Badge>
            </div>
        );
    }

    return (
        <div>
            <Badge fontSize='0.8em' size="large" colorScheme="gray" variant="outline">
                <div className="flex flex-row gap-1 justify-center items-center p-1" >
                    <TimeIcon w={4} h={4} color="gray" />
                    <div>Queued</div>
                </div>
            </Badge>
        </div>
    );
}
export default TaskStatusBadge;
