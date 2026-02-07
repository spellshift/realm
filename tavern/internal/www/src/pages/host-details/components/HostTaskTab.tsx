import { useParams } from "react-router-dom";

import { TablePagination, TableWrapper } from "../../../components/tavern-base-ui/table";
import { TableRowLimit } from "../../../utils/enums";
import TaskCard from "../../../components/task-card/TaskCard";
import { TaskNode } from "../../../utils/interfacesQuery";
import { useHostTasks } from "../useHostTasks";

const HostTaskTab = () => {
    const { hostId } = useParams();

    const {
        data,
        loading,
        initialLoading,
        error,
        updateTaskList,
        page,
        setPage
    } = useHostTasks(hostId);

    return (
        <TableWrapper
            title="Tasks"
            totalItems={data?.tasks?.totalCount}
            loading={initialLoading}
            error={error}
            table={
                <div className="w-full flex flex-col gap-4 my-4">
                    {data?.tasks?.edges.map((task: { node: TaskNode }) => {
                        return (
                            <TaskCard key={task.node.id} task={task.node} />
                        )
                    })}
                </div>
            }
            pagination={
                <TablePagination
                    totalCount={data?.tasks?.totalCount || 0}
                    pageInfo={data?.tasks?.pageInfo || { hasNextPage: false, hasPreviousPage: false, startCursor: null, endCursor: null }}
                    refetchTable={updateTaskList}
                    page={page}
                    setPage={setPage}
                    rowLimit={TableRowLimit.TaskRowLimit}
                    loading={loading}
                />
            }
        />
    );
}
export default HostTaskTab;
