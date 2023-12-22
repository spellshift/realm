import { Badge, Tooltip, Icon } from "@chakra-ui/react";
import { BookOpenIcon } from "@heroicons/react/20/solid";
import { CheckCircleIcon } from "@heroicons/react/24/outline";

import { ColumnDef } from "@tanstack/react-table";
import React, { useCallback } from "react";

import Table from "./tavern-base-ui/Table";
import { TaskStatus } from "../utils/enums";
import { formatDistance } from "date-fns";
import TaskStatusBadge from "./TaskStatusBadge";

type Props = {
    tasks: Array<any>,
    onToggle: (e:any ) => void;
}

const TaskTable = (props: Props) => {
    const {tasks, onToggle} = props;
    const currentDate = new Date();


    const columns: ColumnDef<any>[] = [
        {
            id: "name",
            header: 'Quest details',
            accessorFn: row => row.quest,
            footer: props => props.column.id,
            cell: (cellData: any) => {
                const questData = cellData.getValue();
                return (
                    <div className="flex flex-col">
                        <div>{questData.name}</div>
                        <div className="text-sm flex flex-row gap-1 items-center text-gray-500">
                            {questData?.tome?.name}
                        </div>
                    </div>
                );
            }
        },
        {
            id: "beacon",
            header: 'Beacon',
            accessorFn: row => row.beacon,
            footer: props => props.column.id,
            minSize: window.innerWidth/8,
            cell: (cellData: any) => {
                const beaconData = cellData.getValue();
                return (
                    <div className="flex flex-col gap-1">
                        <div>{beaconData.name}</div>
                        <div className="flex flex-row flex-wrap gap-1">
                            {beaconData?.host?.tags.map((tag: any)=> {
                                return <Badge>{tag.name}</Badge>
                            })}
                            <Badge>{beaconData?.host?.name}</Badge>
                            <Badge>{beaconData?.host?.primaryIP}</Badge>
                            <Badge>{beaconData?.host?.platform}</Badge>
                        </div>
                    </div>
                );
            }
        },
        {
            id: "status",
            header: 'Status',
            accessorFn: row => row,
            maxSize: 100,
            cell: (cellData: any) => {
                const taskData = cellData.getValue();
                const statusTime = new Date(taskData?.lastModifiedAt)
                const hasOutput = taskData?.output.length > 0;
                return (
                    <Tooltip label={taskData?.output.length > 500 ? "Click to see output" : taskData?.output} aria-label='Task output'>
                        <div className="flex flex-col gap-1">
                            <div className="flex flex-row gap-2 flex-wrap">
                                <TaskStatusBadge task={taskData} />
                                {hasOutput && <div>
                                    <Badge fontSize='0.8em' size="large" colorScheme="gray">
                                        <div className="p-1">
                                            Has Output
                                        </div>
                                    </Badge>
                                </div>
                                }
                            </div>
                            <div className="text-sm text-gray-500 flex flex-row flex-wrap">
                                last updated {formatDistance(statusTime, currentDate)}
                            </div>
                        </div>
                    </Tooltip>
                );
            },
            footer: props => props.column.id,
        },

    ];

    return (
        <Table data={tasks} columns={columns} onRowClick={onToggle}/>
    );
}
export default TaskTable;