import { FC } from "react";
import { Task } from "../../../utils/consts";
import { CodeBlock, tomorrow } from "react-code-blocks";
import { BookOpenIcon } from "@heroicons/react/24/outline";

interface TaskResultsType extends Pick<Task, 'error' | 'output' | 'quest'> { };

const TaskResults: FC<TaskResultsType> = ({
    error,
    output,
    quest
}) => {
    return (
        <div className="flex flex-row gap-4">
            <BookOpenIcon className="h-5 w-5 mt-1" />
            <div className="flex flex-col gap-1 w-full">
                <div className="text-gray-600">
                    Tome: {quest?.tome?.name}
                </div>
                <div className="flex flex-col gap-2 text-sm max-h-80 overflow-y-scroll overflow-x-scroll">
                    <div className="max-w-fit">
                        {error ? (
                            <CodeBlock
                                className="-ml-2"
                                text={error}
                                language={""}
                                showLineNumbers={false}
                                theme={tomorrow}
                                codeBlock
                            />
                        ) : output && output?.length > 0 ? (
                            <CodeBlock
                                className="-ml-2"
                                text={output}
                                language={""}
                                showLineNumbers={false}
                                theme={tomorrow}
                                codeBlock
                            />
                        ) : (
                            <div className="mt-2 text-gray-600">No output available</div>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
};
export default TaskResults;
