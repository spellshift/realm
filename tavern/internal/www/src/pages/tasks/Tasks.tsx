import React from "react";
import { Link, useParams } from "react-router-dom";
import { PageWrapper } from "../../features/page-wrapper";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import TablePagination from "../../components/tavern-base-ui/TablePagination";
import { DEFAULT_QUERY_TYPE, PageNavItem, TableRowLimit } from "../../utils/enums";
import { useTasks } from "../../hooks/useTasks";
import { Task } from "../../utils/consts";
import { EditablePageHeader } from "./EditablePageHeader";
import Button from "../../components/tavern-base-ui/button/Button";
import TaskCard from "../../features/task-card/TaskCard";
import FilterControls, { FilterPageType } from "../../components/filter-controls";

const Tasks = () => {
    const { questId } = useParams();
    const pageType = questId ? DEFAULT_QUERY_TYPE.questIdQuery : DEFAULT_QUERY_TYPE.questDetailsQuery;
    const {
        data,
        loading,
        error,
        updateTaskList,
        page,
        setPage
    } = useTasks(pageType, questId);

    return (
        <PageWrapper currNavItem={PageNavItem.quests}>
            <EditablePageHeader />
            <div className="flex flex-row justify-between items-end px-4 py-2 border-b border-gray-200 pb-5">
                <h3 className="text-xl font-semibold leading-6 text-gray-900">{data?.tasks?.edges[0]?.node?.quest.name || questId}</h3>
                <div className="flex flex-row justify-end">
                    {/* Sorting not added yet */}
                    {/* <Button leftIcon={<Bars3BottomLeftIcon className="w-4" />} buttonVariant="ghost" buttonStyle={{ color: 'gray', size: "md" }} onClick={() => console.log("hi")}>Sort</Button> */}
                    <FilterControls type={FilterPageType.TASK} />
                </div>
            </div>
            {loading ? (
                <EmptyState type={EmptyStateType.loading} label="Loading quest tasks..." />
            ) : error ? (
                <EmptyState type={EmptyStateType.error} label="Error loading tasks..." />
            ) : (
                <div>
                    {data?.tasks?.edges.length > 0 ? (
                        <div>
                            <div className=" w-full flex flex-col gap-4 my-4">
                                {data.tasks.edges.map((task: { node: Task }) => {
                                    return (
                                        <TaskCard key={task.node.id} task={task.node} />
                                    )
                                })}
                            </div>
                            <TablePagination totalCount={data?.tasks?.totalCount} pageInfo={data?.tasks?.pageInfo} refetchTable={updateTaskList} page={page} setPage={setPage} rowLimit={TableRowLimit.TaskRowLimit} />
                        </div>
                    ) : (
                        <EmptyState label="No data found" details="Try creating a new quest or adjusting filters." type={EmptyStateType.noData}>
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
        </PageWrapper>
    );
};
export default Tasks;
