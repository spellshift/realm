import React from "react";
import { formatDistance } from 'date-fns'
import { Task, TomeTag } from "../../../utils/consts";

import {
    ColumnDef,
  } from '@tanstack/react-table'
import Table from "../../../components/tavern-base-ui/Table";
import { TaskStatus } from "../../../utils/enums";
import {  RepeatClockIcon, CheckCircleIcon, TimeIcon, WarningIcon } from "@chakra-ui/icons";
import { Tooltip } from '@chakra-ui/react'

type StatusRow = {
    status: TaskStatus,
    time: string,
}

type Props = {
    tasks: Array<Task>;
    onToggle: (e:any ) => void;
};
export const TaskTable = (props: Props) => {
    const {tasks, onToggle} = props;


    const currentDate = new Date();

    const statusValueCompared = (row: Task): number => {
        if(row.execFinishedAt){
            return 1;
        }
        else if(row.execStartedAt){
            return 2;
        }
        else{
            return 3;
        }
    }

    const sortingRow = (rowA: Task, rowB: Task) => {
        return statusValueCompared(rowA) - statusValueCompared(rowB);
    }

    const getStatusDetails = (row: Task) : StatusRow => {
        return row.execFinishedAt ? {status:TaskStatus.finished, time: row.execFinishedAt} : row.execStartedAt ? {status:TaskStatus.inprogress, time: row.execStartedAt} : {status:TaskStatus.queued, time: row.createdAt}
    }

    const getStatusRow = (statusDetails: StatusRow, currentDate: Date) => {
        const statusTime = new Date(statusDetails.time)
        switch (statusDetails.status) {
            case TaskStatus.finished:
                return (
                    <div className="flex flex-col gap-2">
                        <div className="flex flex-row gap-2 items-center"><CheckCircleIcon className="w-8" color="green"/> Finished</div>
                        <div className=" font-light pl-6">
                            {formatDistance(statusTime, currentDate)} ago since finished
                        </div>
                    </div>
                );
            case TaskStatus.inprogress:
                return (
                    <div className="flex flex-col gap-2">
                        <div className="flex flex-row gap-2 items-center">
                            <RepeatClockIcon className="w-8" /> In-Progress
                        </div>
                        <div className=" font-light pl-6">
                            {formatDistance(statusTime, currentDate)} ago since started
                        </div>
                    </div>);
            case TaskStatus.queued:
                return (
                <div className="flex flex-col gap-2">
                    <div className="flex flex-row gap-2 items-center"><TimeIcon className="w-8" />Queued</div>
                    <div className=" font-light pl-6">
                    {formatDistance(statusTime, currentDate)} ago since created
                    </div>
                </div>
                );
            default:
                return  (
                    <div className="flex flex-row gap-2 items-center"><WarningIcon className="w-4" color="red" />Error</div>
            );
        }
    }

    const sortedTasks = [...tasks].sort( (taskA, taskB) => sortingRow(taskA, taskB));

    function getStatusValue(statusRow: StatusRow){
        switch(statusRow.status){
            case TaskStatus.queued:
                return 1;
            case TaskStatus.inprogress:
                return 2;
            case TaskStatus.finished:
                return 3;
            default:
                return 0;
        }
    }

    const columns: ColumnDef<any>[] = [
            {
                id: "status",
                header: 'Status',
                accessorFn: row => getStatusDetails(row),
                cell: (row: any) => getStatusRow(row.getValue(), currentDate),
                footer: props => props.column.id,
                sortingFn: (
                    rowA,
                    rowB,
                    columnId
                  ) => {
                    const statusA = getStatusValue(rowA.getValue(columnId));
                    const statusB = getStatusValue(rowB.getValue(columnId));
                
                    return statusA < statusB ? 1 : statusA > statusB ? -1 : 0;
                  }
            },
            {
                accessorKey: 'beacon.name',
                header: 'Beacon name',
                footer: props => props.column.id,
            },
            {
                id: "Service",
                header: 'Service',
                accessorFn: row => row.beacon?.host?.tags.find((tag: TomeTag) => tag.kind === "service")?.name,
                footer: props => props.column.id,
            },
            {
                id: "group",
                header: 'group',
                accessorFn: row => row.beacon?.host?.tags.find((tag: TomeTag) => tag.kind === "group")?.name,
                footer: props => props.column.id,
            },
            {
                accessorKey: 'output',
                header: 'output',
                cell: (cellData: any) => {
                    const output = cellData.getValue();
                    return (
                        <Tooltip label={output.length > 500 ? "Click to see output" : output} aria-label='Task output'>
                            <div>
                            {output?.substring(0,50)}{output.length > 50 && "..."}
                            </div>
                        </Tooltip>
                    );
                },
                footer: props => props.column.id,
            },
    ];

    return (
        <Table data={sortedTasks} columns={columns} onRowClick={onToggle} />
    )
}