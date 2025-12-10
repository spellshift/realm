import { useCallback } from "react";
import { ColumnDef, Row } from "@tanstack/react-table";
import { formatDistance } from "date-fns";
import { useNavigate } from "react-router-dom";

import Table from "../../../components/tavern-base-ui/Table";
import UserImageAndName from "../../../components/UserImageAndName";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import { QuestEdge, QuestNode, UserNode } from "../../../utils/interfacesQuery";

type Props = {
    quests: QuestEdge[];
}
export const QuestTable = (props: Props) => {
    const { quests } = props;
    const navigate = useNavigate();

    const currentDate = new Date();

    const onToggle = useCallback((row: Row<QuestEdge>) => {
        const questId = row.original?.node?.id;
        if (!questId) return;
        navigate(`/tasks/${questId}`);
    }, [navigate]);

    const columns: ColumnDef<QuestEdge>[] = [
        {
            id: "name",
            header: 'Quest details',
            accessorFn: row => row?.node as QuestNode,
            footer: props => props.column.id,
            enableSorting: false,
            minSize: 200,
            cell: (cellData) => {
                const questData = cellData.getValue() as QuestNode | undefined;
                if (!questData) return null;

                return (
                    <div className="flex flex-col">
                        <div>{questData.name ?? 'N/A'}</div>
                        <div className="text-sm flex flex-row gap-1 items-center text-gray-500 break-all">
                            {questData.tome?.name ?? 'N/A'}
                        </div>
                    </div>
                );
            },
        },
        {
            id: "lastModifiedAt",
            header: 'Updated',
            maxSize: 100,
            accessorFn: row => {
                const lastUpdated = row.node?.lastUpdatedTask?.edges?.[0]?.node?.lastModifiedAt;
                if (!lastUpdated) return 'N/A';
                try {
                    return formatDistance(new Date(lastUpdated), currentDate);
                } catch {
                    return 'Invalid date';
                }
            },
            footer: props => props.column.id,
            enableSorting: false,
        },
        {
            id: "tasksFinished",
            header: 'Finished',
            accessorFn: row => row as QuestEdge,
            maxSize: 60,
            cell: (cellData) => {
                const rowData = cellData.getValue() as QuestEdge | undefined;
                const finished = rowData?.node?.tasksFinished?.totalCount ?? 0;
                const allTasks = rowData?.node?.tasksTotal?.totalCount ?? 0;
                const colorScheme = finished < allTasks ? "none" : "green";

                return (
                    <Badge badgeStyle={{ color: colorScheme }}>
                        {finished}/{allTasks}
                    </Badge>
                );
            },
            footer: (props) => props.column.id,
            enableSorting: false,
        },
        {
            id: "tasksOutput",
            header: 'Output',
            accessorKey: "tasksOutput",
            accessorFn: row => row?.node?.tasksOutput?.totalCount ?? 0,
            maxSize: 60,
            cell: (cellData) => {
                const output = (cellData.getValue() as number | undefined) ?? 0;

                const colorScheme = output === 0 ? "none" : 'purple';

                return (
                    <Badge badgeStyle={{ color: colorScheme }}>
                        {output}
                    </Badge>
                );
            },
            footer: (props) => props.column.id,
            enableSorting: false,
        },
        {
            id: "tasksError",
            header: 'Error',
            accessorFn: row => row?.node?.tasksError?.totalCount ?? 0,
            maxSize: 60,
            cell: (cellData) => {
                const error = (cellData.getValue() as number | undefined) ?? 0;
                const colorScheme = error === 0 ? "none" : 'red';

                return (
                    <Badge badgeStyle={{ color: colorScheme }}>
                        {error}
                    </Badge>
                );
            },
            footer: (props) => props.column.id,
            enableSorting: false,
        },
        {
            id: "creator",
            header: 'Creator',
            maxSize: 100,
            accessorFn: row => row.node?.creator as UserNode,
            footer: props => props.column.id,
            enableSorting: false,
            cell: (cellData) => {
                const creatorData = cellData.getValue() as UserNode;
                return <UserImageAndName userData={creatorData} />
            }
        },
    ];

    return (
        <Table data={quests} columns={columns} onRowClick={onToggle} />
    )
}
