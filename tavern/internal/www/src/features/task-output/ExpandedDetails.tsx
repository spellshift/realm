import { useQuery } from "@apollo/client";
import { CopyBlock, tomorrow } from "react-code-blocks";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import { GET_TASK_DETAILS_QUERY } from "../../utils/queries";
import ShellTable from "./ShellTable";

const ExpandedDetails = ({ id }: { id: string }) => {
    const PARAMS = {
        variables: {
            "where": {
                "id": id
            }
        }
    }
    const { loading, error, data } = useQuery(GET_TASK_DETAILS_QUERY, PARAMS);
    const taskSelected = data?.tasks?.edges?.length > 0 && data?.tasks?.edges[0]?.node || null;
    const output = taskSelected?.output || "No output available";


    if (loading) {
        return <EmptyState type={EmptyStateType.loading} label="Loading task details..." />
    }
    if (error) {
        return <EmptyState type={EmptyStateType.error} label="Error loading task details..." />
    }


    return (
        <>
            {taskSelected && taskSelected.shells.length > 0 && (
                <div className="flex flex-col gap-2">
                    <h3 className="text-2xl text-gray-800">Shells</h3>
                    <ShellTable shells={taskSelected.shells} />
                </div>
            )}
            <div className="flex flex-col gap-2">
                <h3 className="text-2xl text-gray-800">Output</h3>
                <div className="bg-gray-200 rounded-md p-0.5 ">
                    <CopyBlock
                        text={output}
                        language={""}
                        showLineNumbers={false}
                        theme={tomorrow}
                        codeBlock
                    />
                </div>
            </div>
        </>
    );
}
export default ExpandedDetails;
