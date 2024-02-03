import { useState } from "react";
import { Link, useParams } from "react-router-dom";
import { TaskOutput } from "../../../features/task-output";
import TaskTable from "../../../components/TaskTable";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import TablePagination from "../../../components/tavern-base-ui/TablePagination";
import { TableRowLimit } from "../../../utils/enums";
import FreeTextSearch from "../../../components/tavern-base-ui/DebouncedFreeTextSearch";
import { TASK_PAGE_TYPE, useTasks } from "../../../hooks/useTasks";

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
    } = useTasks(TASK_PAGE_TYPE.hostIDQuery, hostId);

    const [isOpen, setOpen] = useState(false);
    const [selectedTask, setSelectedTask] = useState<any | null>(null);

    const handleClick = (e: any) => {
        const selectedTaskData = e?.original?.node;
        setSelectedTask(selectedTaskData);
        setOpen((state) => !state);
    }

    return (
        <>
            <div className="py-4 bg-white rounded-lg shadow-lg mt-2 flex flex-col gap-2">
                <div className="px-6 pt-2 ">
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
                                <TaskTable tasks={taskData?.tasks?.edges} onToggle={handleClick} />
                                <TablePagination totalCount={taskData?.tasks?.totalCount} pageInfo={taskData?.tasks?.pageInfo} refetchTable={updateTaskList} page={page} setPage={setPage} rowLimit={TableRowLimit.TaskRowLimit} />
                            </>
                        )
                            : (
                                <EmptyState label="No tasks found" type={EmptyStateType.noData} details="Get started by creating a new quest." >
                                    <Link to="/createQuest">
                                        <button
                                            type="button"
                                            className="inline-flex items-center rounded-md bg-purple-700 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-purple-600 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-purple-700"
                                        >
                                            Create new quest
                                        </button>
                                    </Link>
                                </EmptyState>
                            )}
                    </div>
                )}
            </div>
            <TaskOutput isOpen={isOpen} setOpen={setOpen} selectedTask={selectedTask} />
        </>
    );
}
export default HostTasks;
