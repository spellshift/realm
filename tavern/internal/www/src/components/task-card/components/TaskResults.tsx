import CodeBlock from "../../tavern-base-ui/CodeBlock";

const TaskResults = ({
    result
}: { result?: string | null }) => {
    return (
        <div className="text-sm max-h-80 overflow-y-scroll overflow-x-scroll py-1">
            <div className="max-w-fit">
                {result && result.length > 0 ? (
                    <CodeBlock code={result} />
                ) : (
                    <div className="mt-2 text-gray-600">Not available</div>
                )}
            </div>
        </div>
    );
};
export default TaskResults;
