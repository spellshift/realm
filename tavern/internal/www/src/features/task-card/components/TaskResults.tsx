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
                                text={output + "Where can I get some? There are many variations of passages of Lorem Ipsum available, but the majority have suffered alteration in some form, by injected humour, or randomised words which don't look even slightly believable. If you are going to use a passage of Lorem Ipsum, you need to be sure there isn't anything embarrassing hidden in the middle of text. All the Lorem Ipsum generators on the Internet tend to repeat predefined chunks as necessary, making this the first true generator on the Internet. It uses a dictionary of over 200 Latin words, combined with a handful of model sentence structures, to generate Lorem Ipsum which looks reasonable. The generated Lorem Ipsum is therefore always free from repetition, injected humour, or non-characteristic words etc."}
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
