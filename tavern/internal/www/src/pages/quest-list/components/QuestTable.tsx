import { useCallback } from "react";
import { ColumnDef } from "@tanstack/react-table";
import { formatDistance } from "date-fns";
import { useNavigate } from "react-router-dom";

import Table from "../../../components/tavern-base-ui/Table";
import { QuestTableRowType } from "../../../utils/consts";
import UserImageAndName from "../../../components/UserImageAndName";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import { useFilters } from "../../../context/FilterContext";



type Props = {
    quests: Array<QuestTableRowType>;
}
export const QuestTable = (props: Props) => {
    const { filters } = useFilters();
    const { quests } = props;
    const navigate = useNavigate();

    const currentDate = new Date();

    const onToggle = useCallback((row: any) => {
        navigate(`/tasks/${row?.original?.node?.id}`, {
            state: filters
        })
    }, [filters, navigate]);

    const columns: ColumnDef<any>[] = [
        {
            id: "name",
            header: 'Quest details',
            accessorFn: row => row?.node,
            footer: props => props.column.id,
            enableSorting: false,
            minSize: 200,
            cell: (cellData: any) => {
                const questData = cellData.getValue();
                return (
                    <div className="flex flex-col">
                        <div>{questData?.name}</div>
                        <div className="text-sm flex flex-row gap-1 items-center text-gray-500 break-all">
                            {questData?.tome?.name}
                        </div>
                    </div>
                );
            },
        },
        {
            id: "lastModifiedAt",
            header: 'Updated',
            maxSize: 100,
            accessorFn: row => formatDistance(new Date(row?.node?.lastUpdatedTask?.edges[0].node.lastModifiedAt), currentDate),
            footer: props => props.column.id,
            enableSorting: false,
        },
        {
            id: "tasksFinished",
            header: 'Finished',
            accessorFn: row => row,
            maxSize: 60,
            cell: (row: any) => {
                const rowData = row.getValue();
                const finished = rowData?.node?.tasksFinished?.totalCount || 0;
                const allTasks = rowData?.node?.tasksTotal?.totalCount || 0;
                const colorScheme = finished < allTasks ? "none" : "green";

                return (
                    <Badge badgeStyle={{ color: colorScheme }}>
                        {finished}/{allTasks}
                    </Badge>
                );
            },
            footer: (props: any) => props.column.id,
            enableSorting: false,
        },
        {
            id: "tasksOutput",
            header: 'Output',
            accessorKey: "tasksOutput",
            accessorFn: row => row?.node?.tasksOutput?.totalCount,
            maxSize: 60,
            cell: (cellData: any) => {
                const output = cellData.getValue();

                const colorScheme = output === 0 ? "none" : 'purple';

                return (
                    <Badge badgeStyle={{ color: colorScheme }}>
                        {output}
                    </Badge>
                );
            },
            footer: (props: any) => props.column.id,
            enableSorting: false,
        },
        {
            id: "tasksError",
            header: 'Error',
            accessorFn: row => row?.node?.tasksError?.totalCount,
            maxSize: 60,
            cell: (cellData: any) => {
                const error = cellData.getValue();
                const colorScheme = error === 0 ? "none" : 'red';

                return (
                    <Badge badgeStyle={{ color: colorScheme }}>
                        {error}
                    </Badge>
                );
            },
            footer: (props: any) => props.column.id,
            enableSorting: false,
        },
        {
            id: "creator",
            header: 'Creator',
            maxSize: 100,
            accessorFn: row => row.node?.creator,
            footer: props => props.column.id,
            enableSorting: false,
            cell: (cellData: any) => {
                const creatorData = cellData.getValue();
                return <UserImageAndName userData={creatorData} />
            }
        },
    ];

    return (
        <Table data={quests} columns={columns} onRowClick={onToggle} />
    )
}
