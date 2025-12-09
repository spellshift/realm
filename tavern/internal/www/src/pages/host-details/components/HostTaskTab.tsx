import { Link, useParams } from "react-router-dom";

import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import TablePagination from "../../../components/tavern-base-ui/TablePagination";
import { DEFAULT_QUERY_TYPE, PageNavItem, TableRowLimit } from "../../../utils/enums";
import Button from "../../../components/tavern-base-ui/button/Button";
import TaskCard from "../../../components/task-card/TaskCard";
import FilterControls, { FilterPageType } from "../../../components/FilterControls";
import { useTasks } from "../../../hooks/useTasks";
import { TaskNode } from "../../../utils/interfacesQuery";
import SortingControls from "../../../components/SortingControls";

const HostTaskTab = () => {
    const { hostId } = useParams();

    const {
        data,
        loading,
        error,
        updateTaskList,
        page,
        setPage
    } = useTasks(DEFAULT_QUERY_TYPE.hostIDQuery, hostId);

    return (
        <div className="flex flex-col gap-2 mt-4">
            <div className="flex flex-row justify-end">
                <SortingControls type={PageNavItem.tasks} />
                <FilterControls type={FilterPageType.HOST_TASK} />
            </div>
            {loading ? (
                <EmptyState type={EmptyStateType.loading} label="Loading tasks..." />
            ) : error ? (
                <EmptyState type={EmptyStateType.error} label="Error loading tasks..." />
            ) : (
                <div>
                    {data?.tasks?.edges.length > 0 ? (
                        <>
                            <div className=" w-full flex flex-col gap-2 my-4">
                                {data.tasks.edges.map((task: { node: TaskNode }) => {
                                    return (
                                        <TaskCard key={task.node.id} task={task.node} />
                                    )
                                })}
                            </div>
                            <TablePagination totalCount={data?.tasks?.totalCount} pageInfo={data?.tasks?.pageInfo} refetchTable={updateTaskList} page={page} setPage={setPage} rowLimit={TableRowLimit.TaskRowLimit} />
                        </>
                    )
                        : (
                            <EmptyState label="No data found" type={EmptyStateType.noData} details="Try creating a new quest or adjusting filters." >
                                <Link to="/createQuest">
                                    <Button
                                        buttonStyle={{ color: "purple", "size": "md" }}
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
export default HostTaskTab;
