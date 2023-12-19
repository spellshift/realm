import React, { useState } from "react";
import { Link } from "react-router-dom";
import { PageWrapper } from "../../components/page-wrapper";
import { TaskOutput } from "../../components/task-output";
import TaskTable from "../../components/TaskTable";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import FilterBar from "./FilterBar";
import { useTasks } from "./useTasks";

const Tasks = () => {

    const {
        data,
        loading,
        error,
        setSearch,
        setGroups,
        setBeacons,
        setServices,
        setHosts,
        setPlatforms
    } = useTasks();

    const [isOpen, setOpen] = useState(false);
    const [selectedTask, setSelectedTask] = useState<any | null>(null);

    const handleClick =(e: any) => {
        const selectedTaskData = e?.original;
        console.log(selectedTask);
        setSelectedTask(selectedTaskData);
        setOpen((state)=> !state);
    }

    // TODO: REMOVE THIS LIMIT
    const tableData = data?.tasks.slice(0,100);

    return (
        <PageWrapper>
            <div className="border-b border-gray-200 pb-5 sm:flex sm:items-center sm:justify-between">
                <h3 className="text-xl font-semibold leading-6 text-gray-900">Quest outputs</h3>
            </div>
                {loading ? (
                    <EmptyState type={EmptyStateType.loading} label="Loading quest tasks..." />
                ) : error ? (
                    <EmptyState type={EmptyStateType.error} label="Error loading taks..." />
                ) : (
                    <div>
                        <FilterBar setSearch={setSearch} setBeacons={setBeacons} setGroups={setGroups} setServices={setServices} setHosts={setHosts} setPlatforms={setPlatforms} />
                        {data?.tasks?.length > 0 ? (
                            <div className="py-4 bg-white rounded-lg shadow-lg mt-2">
                                <TaskTable tasks={tableData} onToggle={handleClick} />
                                <div className="px-4">
                                    * Table only shows top 100 results matching query
                                </div>
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