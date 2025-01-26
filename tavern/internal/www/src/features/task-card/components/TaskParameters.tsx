import { FC } from "react";
import { QuestProps, TomeParams } from "../../../utils/consts";
import { constructTomeParams } from "../../../utils/utils";
import { WrenchScrewdriverIcon } from "@heroicons/react/24/outline";

interface TaskParametersType {
    quest?: QuestProps
}
const TaskParameters: FC<TaskParametersType> = ({
    quest
}) => {
    const params = constructTomeParams(quest?.parameters, quest?.tome?.paramDefs);

    return (
        <div className="flex flex-row gap-4">
            <WrenchScrewdriverIcon className="h-5 w-5 mt-1" />
            <div className="flex flex-col gap-1 ">
                <div className="text-gray-600">
                    Tome parameters
                </div>
                {params.map((paramDef: TomeParams) => {
                    if (paramDef.value) {
                        return (
                            <div className="flex flex-row gap-1 text-xs" key={paramDef.name}>
                                <div className="font-semibold">
                                    {paramDef.name}:
                                </div>
                                <div>
                                    {paramDef.value}
                                </div>
                            </div>
                        )
                    }
                    else {
                        return null;
                    }
                })}
            </div>
        </div>
    );
};
export default TaskParameters;
