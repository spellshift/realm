import { Badge, Tooltip } from "@chakra-ui/react";

import { ColumnDef } from "@tanstack/react-table";
import React from "react";

import Table from "./tavern-base-ui/Table";
import { formatDistance } from "date-fns";
import TaskStatusBadge from "./TaskStatusBadge";
import BeaconTile from "./BeaconTile";

type Props = {
    tasks: Array<any>,
    onToggle: (e: any) => void;
}

const TaskTable = (props: Props) => {
    const { tasks, onToggle } = props;
    const currentDate = new Date();


    const columns: ColumnDef<any>[] = [
        {
            id: "name",
            header: 'Quest details',
            accessorFn: row => row?.node?.quest,
            footer: props => props.column.id,
            maxSize: 110,
            enableSorting: false,
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
            accessorFn: row => row?.node?.beacon,
            footer: props => props.column.id,
            minSize: window.innerWidth / 8,
            enableSorting: false,
            cell: (cellData: any) => {
                const beaconData = cellData.getValue();
                return (
                    <BeaconTile beaconData={beaconData} />
                );
            }
        },
        {
            id: "status",
            header: 'Status',
            accessorFn: row => row?.node,
            minSize: 80,
            maxSize: 100,
            enableSorting: false,
            cell: (cellData: any) => {
                const taskData = cellData.getValue();
                const statusTime = new Date(taskData?.lastModifiedAt)
                const hasOutput = taskData?.outputSize > 0;
                const toolTipText = taskData?.outputSize > 0 ? `Click to view (${taskData.outputSize} characters of output)` : 'Click to view';
                return (
                    <Tooltip label={toolTipText} aria-label='Task output'>
                        <div className="flex flex-col gap-1">
                            <div className="flex flex-row gap-2 flex-wrap">
                                <TaskStatusBadge task={taskData} />
                                {hasOutput && <div>
                                    <Badge fontSize='0.8em' size="large" colorScheme="purple">
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
        <Table data={tasks} columns={columns} onRowClick={onToggle} />
    );
}
export default TaskTable;
