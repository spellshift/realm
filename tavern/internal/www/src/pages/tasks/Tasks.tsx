import { useParams } from "react-router-dom";
import { PageWrapper } from "../../components/page-wrapper";
import { TablePagination, TableWrapper } from "../../components/tavern-base-ui/table";
import { PageNavItem, TableRowLimit } from "../../utils/enums";
import { useTasks } from "./useTasks";
import TaskCard from "../../components/task-card/TaskCard";
import { FilterControls, FilterPageType } from "../../context/FilterContext/index";
import { TaskEdge } from "../../utils/interfacesQuery";
import { EditablePageHeader } from "./EditablePageHeader";
import { SortingControls } from "../../context/SortContext/index";

const Tasks = () => {
    const { questId } = useParams();
    const {
        data,
        loading,
        error,
        updateTaskList,
        page,
        setPage
    } = useTasks(questId);

    return (
        <PageWrapper currNavItem={PageNavItem.quests}>
            <EditablePageHeader />
            <TableWrapper
                title="Tasks"
                totalItems={data?.tasks?.totalCount}
                loading={loading}
                error={error}
                filterControls={<FilterControls type={FilterPageType.TASK} />}
                sortingControls={<SortingControls type={PageNavItem.tasks} />}
                table={
                    <div className="w-full flex flex-col gap-4 my-4">
                        {data?.tasks?.edges.map((task: TaskEdge) => {
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
                    />
                }
            />
        </PageWrapper>
    );
};
export default Tasks;
