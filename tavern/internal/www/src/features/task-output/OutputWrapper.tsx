import { useQuery } from "@apollo/client";
import { CopyBlock, tomorrow } from "react-code-blocks";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import { GET_TASK_OUTPUT_QUERY } from "../../utils/queries";

const OutputWrapper = ({ id }: { id: string }) => {
    const PARAMS = {
        variables: {
            "where": {
                "id": id
            }
        }
    }
    const { loading, error, data } = useQuery(GET_TASK_OUTPUT_QUERY, PARAMS);
    const output = (data?.tasks?.edges?.length > 0 && data?.tasks?.edges[0]?.node?.output) ? data?.tasks?.edges[0]?.node?.output : "No output available";
    return (
        <div className="flex flex-col gap-2">
            <h3 className="text-2xl text-gray-800">Output</h3>
            {loading ? (
                <EmptyState type={EmptyStateType.loading} label="Loading tasks..." />
            ) : error ? (
                <EmptyState type={EmptyStateType.error} label="Error loading tasks..." />
            ) : (
                <div className="bg-gray-200 rounded-md p-0.5 ">
                    <CopyBlock
                        text={output}
                        language={""}
                        showLineNumbers={false}
                        theme={tomorrow}
                        codeBlock
                    />
                </div>
            )}
        </div>
    );
}
export default OutputWrapper;
