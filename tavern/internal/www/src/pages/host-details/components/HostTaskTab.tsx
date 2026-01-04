import { useParams } from "react-router-dom";

import { TablePagination, TableWrapper } from "../../../components/tavern-base-ui/table";
import { PageNavItem, TableRowLimit } from "../../../utils/enums";
import TaskCard from "../../../components/task-card/TaskCard";
import { FilterControls, FilterPageType } from "../../../context/FilterContext/index";
import { TaskNode } from "../../../utils/interfacesQuery";
import { SortingControls } from "../../../context/SortContext/index";
import { useHostTasks } from "../useHostTasks";

const HostTaskTab = () => {
    const { hostId } = useParams();

    const {
        data,
        loading,
        error,
        updateTaskList,
        page,
        setPage
    } = useHostTasks(hostId);

    return (
        <TableWrapper
            title="Tasks"
            totalItems={data?.tasks?.totalCount || null}
            loading={loading}
            error={error}
            filterControls={<FilterControls type={FilterPageType.HOST_TASK} />}
            sortingControls={<SortingControls type={PageNavItem.tasks} />}
            table={
                <div className="overflow-x-auto -mx-4 sm:-mx-6 xl:-mx-8">
                    <div className="inline-block min-w-full align-middle flex flex-col gap-2 my-4">
                        {data?.tasks?.edges.map((task: { node: TaskNode }) => {
                            return (
                                <TaskCard key={task.node.id} task={task.node} />
                            )
                        })}
                    </div>
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
                />
            }
        />
    );
}
export default HostTaskTab;
