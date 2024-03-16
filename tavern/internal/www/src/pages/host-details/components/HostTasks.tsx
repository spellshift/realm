import { useState } from "react";
import { Accordion, AccordionButton, AccordionIcon, AccordionItem, AccordionPanel, Box } from "@chakra-ui/react";
import { Link, useParams } from "react-router-dom";

import { TaskOutput } from "../../../features/task-output";
import TaskTable from "../../../components/TaskTable";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import TablePagination from "../../../components/tavern-base-ui/TablePagination";
import { DEFAULT_QUERY_TYPE, TableRowLimit } from "../../../utils/enums";
import FreeTextSearch from "../../../components/tavern-base-ui/DebouncedFreeTextSearch";
import { useTasks } from "../../../hooks/useTasks";
import Button from "../../../components/tavern-base-ui/button/Button";

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

    const [isOpen, setOpen] = useState(false);
    const [selectedTask, setSelectedTask] = useState<any | null>(null);

    const handleClick = (e: any) => {
        const selectedTaskData = e?.original?.node;
        setSelectedTask(selectedTaskData);
        setOpen((state) => !state);
    }

    return (
        <Accordion allowToggle className='w-full' defaultIndex={[0]}>
            <AccordionItem>
                <AccordionButton>
                    <Box as="span" flex='1' textAlign='left'>
                        <h2 className="text-lg font-semibold text-gray-900">Tasks</h2>
                    </Box>
                    <AccordionIcon />
                </AccordionButton>
                <AccordionPanel>
                    <div className="flex flex-col gap-2">
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
                    {isOpen && <TaskOutput isOpen={isOpen} setOpen={setOpen} selectedTask={selectedTask} />}
                </AccordionPanel>
            </AccordionItem>
        </Accordion>
    );
}
export default HostTasks;
