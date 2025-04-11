import { CodeBlock, tomorrow } from "react-code-blocks";

const TaskResults = ({
    result
}: { result?: string }) => {
    return (
        <div className="text-sm max-h-80 overflow-y-scroll overflow-x-scroll py-1">
            <div className="max-w-fit">
                {result && result.length > 0 ? (
                    <div className="-ml-2">
                        <CodeBlock
                            text={result}
                            language={""}
                            showLineNumbers={false}
                            theme={tomorrow}
                            codeBlock
                        />
                    </div>
                ) : (
                    <div className="mt-2 text-gray-600">Not available</div>
                )}
            </div>
        </div>
    );
};
export default TaskResults;
