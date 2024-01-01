import React, { useState } from "react";
import { Link, useParams } from "react-router-dom";
import { PageWrapper } from "../../components/page-wrapper";
import { TaskOutput } from "../../components/task-output";
import TaskTable from "../../components/TaskTable";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import TablePagination from "../../components/tavern-base-ui/TablePagination";
import { TableRowLimit } from "../../utils/enums";
import FilterBar from "./FilterBar";
import { TaskPageHeader } from "./TaskPageHeader";
import { TASK_PAGE_TYPE, useTasks } from "./useTasks";

const Tasks = () => {
    const { questId } = useParams();
    const pageType = questId ? TASK_PAGE_TYPE.questIdQuery : TASK_PAGE_TYPE.questDetailsQuery;
    const {
        data,
        loading,
        error,
        setSearch,
        setFiltersSelected,
        updateTaskList,
        page,
        setPage
    } = useTasks(pageType, questId);

    const [isOpen, setOpen] = useState(false);
    const [selectedTask, setSelectedTask] = useState<any | null>(null);

    const handleClick =(e: any) => {
        const selectedTaskData = e?.original?.node;
        setSelectedTask(selectedTaskData);
        setOpen((state)=> !state);
    }

    return (
        <PageWrapper>
            <div className="border-b border-gray-200 pb-5 sm:flex sm:items-center sm:justify-between">
                <TaskPageHeader />
            </div>
            <FilterBar setSearch={setSearch} setFiltersSelected={setFiltersSelected} />
            {loading ? (
                <EmptyState type={EmptyStateType.loading} label="Loading quest tasks..." />
            ) : error ? (
                <EmptyState type={EmptyStateType.error} label="Error loading tasks..." />
            ) : (
                <div>
                    {data?.tasks?.edges.length > 0 ? (
                        <div className="py-4 bg-white rounded-lg shadow-lg mt-2 flex flex-col gap-1">
                            <TaskTable tasks={data?.tasks?.edges} onToggle={handleClick} />
                            <TablePagination totalCount={data?.tasks?.totalCount} pageInfo={data?.tasks?.pageInfo} refetchTable={updateTaskList} page={page} setPage={setPage} rowLimit={TableRowLimit.TaskRowLimit} />
                        </div>
                    ): (
                        <EmptyState label="No data found" details="Try creating a new quest or adjusting filters." type={EmptyStateType.noData}>
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
            <TaskOutput isOpen={isOpen} setOpen={setOpen} selectedTask={selectedTask}/>
        </PageWrapper>
    );
};
export default Tasks;