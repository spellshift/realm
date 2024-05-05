import { Link, useParams } from "react-router-dom";

import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import TablePagination from "../../../components/tavern-base-ui/TablePagination";
import { DEFAULT_QUERY_TYPE, TableRowLimit } from "../../../utils/enums";
import FreeTextSearch from "../../../components/tavern-base-ui/DebouncedFreeTextSearch";
import { useTasks } from "../../../hooks/useTasks";
import Button from "../../../components/tavern-base-ui/button/Button";
import TaskCard from "../../../features/task-card/TaskCard";
import { Task } from "../../../utils/consts";

const HostTasks = () => {
    const { hostId } = useParams();
    const {
        data: taskData,
        loading: taskLoading,
        error: taskError,
        page,
        setPage,
        setSearch,
        updateTaskList
    } = useTasks(DEFAULT_QUERY_TYPE.hostIDQuery, hostId);

    return (
        <div className="flex flex-col gap-2 mt-4">
            <div className="pt-2 ">
                <FreeTextSearch setSearch={setSearch} />
            </div>
            {taskLoading ? (
                <EmptyState type={EmptyStateType.loading} label="Loading tasks..." />
            ) : taskError ? (
                <EmptyState type={EmptyStateType.error} label="Error loading tasks..." />
            ) : (
                <div>
                    {taskData?.tasks?.edges.length > 0 ? (
                        <>
                            <div className=" w-full flex flex-col gap-2 my-4">
                                {taskData.tasks.edges.map((task: { node: Task }) => {
                                    return (
                                        <TaskCard key={task.node.id} task={task.node} />
                                    )
                                })}
                            </div>
                            <TablePagination totalCount={taskData?.tasks?.totalCount} pageInfo={taskData?.tasks?.pageInfo} refetchTable={updateTaskList} page={page} setPage={setPage} rowLimit={TableRowLimit.TaskRowLimit} />
                        </>
                    )
                        : (
                            <EmptyState label="No tasks found" type={EmptyStateType.noData} details="Get started by creating a new quest." >
                                <Link to="/createQuest">
                                    <Button
                                        type="button"
                                    >
                                        Create new quest
                                    </Button>
                                </Link>
                            </EmptyState>
                        )}
                </div>
            )}
        </div>
    );
}
export default HostTasks;
