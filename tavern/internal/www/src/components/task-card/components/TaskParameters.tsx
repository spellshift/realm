import { FC } from "react";
import { constructTomeParams } from "../../../utils/utils";
import { BookOpenIcon } from "@heroicons/react/24/outline";
import { QuestNode } from "../../../utils/interfacesQuery";
import { FieldInputParams } from "../../../utils/interfacesUI";

interface TaskParametersType {
    quest?: QuestNode | undefined
}
const TaskParameters: FC<TaskParametersType> = ({
    quest
}) => {
    const params = constructTomeParams(quest?.parameters, quest?.tome?.paramDefs);

    return (
        <div className="flex flex-row gap-4">
            <BookOpenIcon className="h-5 w-5 mt-1" />
            <div className="flex flex-col gap-1 ">
                <div className="text-gray-600 break-all">
                    {quest?.tome?.name}
                </div>
                {params.map((paramDef: FieldInputParams) => {
                    if (paramDef.value) {
                        return (
                            <div className="flex flex-row gap-1 text-sm text-gray-600" key={paramDef.name}>
                                <div className="font-semibold">
                                    {paramDef.name}:
                                </div>
                                <div className="break-all">
                                    {paramDef.value}
                                </div>
                            </div>
                        )
                    }
                    else {
                        return <div className="text-sm text-gray-600">Not available</div>;
                    }
                })}
            </div>
        </div>
    );
};
export default TaskParameters;
