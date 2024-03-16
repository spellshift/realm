import React, { useState } from "react";
import { Link, useParams } from "react-router-dom";
import { PageWrapper } from "../../components/page-wrapper";
import { TaskOutput } from "../../features/task-output";
import TaskTable from "../../components/TaskTable";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import TablePagination from "../../components/tavern-base-ui/TablePagination";
import { DEFAULT_QUERY_TYPE, PageNavItem, TableRowLimit } from "../../utils/enums";
import FilterBar from "../../components/FilterBar";
import { useTasks } from "../../hooks/useTasks";
import { Task } from "../../utils/consts";
import { EditablePageHeader } from "./EditablePageHeader";
import { useQuests } from "../../hooks/useQuests";
import Button from "../../components/tavern-base-ui/button/Button";

const Tasks = () => {
    const { questId } = useParams();
    const pageType = questId ? DEFAULT_QUERY_TYPE.questIdQuery : DEFAULT_QUERY_TYPE.questDetailsQuery;
    const {
        data,
        loading,
        error,
        setSearch,
        setFiltersSelected,
        filtersSelected,
        updateTaskList,
        page,
        setPage
    } = useTasks(pageType, questId);

    const {
        data: questData,
        loading: questLoading,
        error: questError,
        setFiltersSelected: setQuestFiltersSelected
    } = useQuests(false, questId);

    const [isOpen, setOpen] = useState(false);
    const [selectedTask, setSelectedTask] = useState<Task | null>(null);

    const handleClick = (e: any) => {
        const selectedTaskData = e?.original?.node;
        setSelectedTask(selectedTaskData);
        setOpen((state) => !state);
    }

    const handleFilterSelected = (filtersSelected: Array<any>) => {
        setFiltersSelected(filtersSelected);
        setQuestFiltersSelected(filtersSelected);
    }

    return (
        <PageWrapper currNavItem={PageNavItem.quests}>
            <div className="border-b border-gray-200 pb-5 sm:flex sm:items-center sm:justify-between">
                <EditablePageHeader questId={questId} data={questData} loading={questLoading} error={questError} />
            </div>
            <div className="p-4 bg-white rounded-lg shadow-lg mt-2">
                <FilterBar setSearch={setSearch} filtersSelected={filtersSelected} setFiltersSelected={handleFilterSelected} />
            </div>
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
                    ) : (
                        <EmptyState label="No data found" details="Try creating a new quest or adjusting filters." type={EmptyStateType.noData}>
                            <Link to="/createQuest">
                                <Button
                                    buttonStyle={{ color: "gray", "size": "md" }}
                                    type="button"
                                >
                                    Create new quest
                                </Button>
                            </Link>
                        </EmptyState>
                    )}
                </div>
            )}
            {isOpen && <TaskOutput isOpen={isOpen} setOpen={setOpen} selectedTask={selectedTask} />}
        </PageWrapper>
    );
};
export default Tasks;
