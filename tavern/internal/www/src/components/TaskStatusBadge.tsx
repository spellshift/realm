import { RepeatClockIcon, TimeIcon } from "@chakra-ui/icons";
import { Badge } from "@chakra-ui/react";
import { CheckCircleIcon } from "@heroicons/react/24/outline";

type Props = {
    task: any;
}
const TaskStatusBadge = (props: Props) => {
    const {task} = props;

    if(task.execFinishedAt){
        return (
            <div>
                <Badge fontSize='0.8em' size="large" colorScheme="green" variant="outline">
                    <div className="flex flex-row gap-1 justify-center items-center p-1" >
                        <CheckCircleIcon className="w-4" color="green"/>
                        <div>Finished</div>
                    </div>
                </Badge>
            </div>
        );
    }

    if(task.execStartedAt){
       return (
        <div>
            <Badge fontSize='0.8em' size="large" colorScheme="gray" variant="outline">
                <div className="flex flex-row gap-1 justify-center items-center p-1" >
                    <RepeatClockIcon className="w-4" color="gray"/> 
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
                    <TimeIcon className="w-4" color="gray" />
                    <div>Queued</div>
                </div>
            </Badge>
        </div>
    );
}
export default TaskStatusBadge;