import CodeBlock from "../../tavern-base-ui/CodeBlock";

const TaskResults = ({
    result
}: { result?: string | null }) => {
    return (
        <div className="text-sm max-h-80 overflow-y-scroll overflow-x-scroll py-1" aria-label="task output">
            {result && result.length > 0 ? (
                <CodeBlock code={result} />
            ) : (
                <div className="py-3 px-2 text-sm">Not available</div>
            )}
        </div>
    );
};
export default TaskResults;
