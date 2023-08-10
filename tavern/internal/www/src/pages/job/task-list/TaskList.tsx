import React, { FC } from "react";
import { formatDistance } from 'date-fns'
import { Task, TomeTag } from "../../../utils/consts";

import {
    useReactTable,
    getCoreRowModel,
    getExpandedRowModel,
    ColumnDef,
    flexRender,
    Row,
  } from '@tanstack/react-table'
import Table from "../../../components/tavern-base-ui/Table";
import { TaskStatus } from "../../../utils/enums";
import {  RepeatClockIcon, CheckCircleIcon, TimeIcon, WarningIcon, ViewIcon  } from "@chakra-ui/icons";

type StatusRow = {
    status: TaskStatus,
    time: string,
}

type Props = {
    tasks: Array<Task>
};
export const TaskList = (props: Props) => {
    const {tasks} = props;

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
                        <div className="flex flex-row gap-2 items-center"><CheckCircleIcon className="w-4" color="green"/> Finished</div>
                        <div className=" font-light pl-6">
                            {formatDistance(statusTime, currentDate)} ago since finished
                        </div>
                    </div>
                );
            case TaskStatus.inprogress:
                return (
                    <div className="flex flex-col gap-2">
                        <div className="flex flex-row gap-2 items-center">
                            <RepeatClockIcon className="w-4" /> In-Progress
                        </div>
                        <div className=" font-light pl-6">
                            {formatDistance(statusTime, currentDate)} ago since started
                        </div>
                    </div>);
            case TaskStatus.queued:
                return (
                <div className="flex flex-col gap-2">
                    <div className="flex flex-row gap-2 items-center"><TimeIcon className="w-4" />Queued</div>
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

    const columns: ColumnDef<any>[] = [
            {
                id: "status",
                header: 'Status',
                accessorFn: row => getStatusDetails(row),
                cell: (row: any) => getStatusRow(row.getValue(), currentDate),
                footer: props => props.column.id,
            },
            {
                accessorKey: 'session.name',
                header: 'Session name',
                footer: props => props.column.id,
            },
            {
                id: "Service",
                header: 'Service',
                accessorFn: row => row.session.tags.find((tag: TomeTag) => tag.kind === "service")?.name,
                footer: props => props.column.id,
            },
            {
                id: "group",
                header: 'group',
                accessorFn: row => row.session.tags.find((tag: TomeTag) => tag.kind === "group")?.name,
                footer: props => props.column.id,
            },
            {
                accessorKey: 'output',
                header: 'output',
                footer: props => props.column.id,
            },
    ];

    return (
        <Table data={sortedTasks} columns={columns} />
    )
}