import { Image } from "@chakra-ui/react";
import { FC } from "react";
import { UserType } from "../../../utils/consts";

interface TaskCreatorType {
    creatorData: UserType | undefined;
}
const TaskCreator: FC<TaskCreatorType> = ({
    creatorData
}) => {

    if (!creatorData) {
        return null;
    }

    return (
        <div className="flex flex-row gap-4">
            <Image
                className="mt-1 w-5"
                borderRadius='full'
                boxSize='18px'
                src={creatorData.photoURL}
                alt={`Profile of ${creatorData.name}`}
            />
            <div className="text-gray-600">
                {creatorData.name}
            </div>
        </div>
    )
}
export default TaskCreator;
