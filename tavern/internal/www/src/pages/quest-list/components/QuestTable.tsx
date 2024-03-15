import { Badge, Image } from "@chakra-ui/react";
import { ColumnDef } from "@tanstack/react-table";
import { formatDistance } from "date-fns";
import Table from "../../../components/tavern-base-ui/Table";
import { useNavigate } from "react-router-dom";
import { QuestTableRowType } from "../../../utils/consts";
import { useCallback } from "react";


type Props = {
    quests: Array<QuestTableRowType>;
    filtersSelected: Array<any>;
}
export const QuestTable = (props: Props) => {
    const { quests, filtersSelected } = props;
    const navigate = useNavigate();

    const currentDate = new Date();

    const onToggle = useCallback((row: any) => {
        navigate(`/tasks/${row?.original?.node?.id}`, {
            state: filtersSelected
        })
    }, [filtersSelected, navigate]);

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
                        <div className="text-sm flex flex-row gap-1 items-center text-gray-500">
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
                const colorScheme = finished < allTasks ? "alphaWhite" : "green";

                return (
                    <Badge ml='1' px='4' colorScheme={colorScheme} fontSize="font-base">
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

                const colorScheme = output === 0 ? "alphaWhite" : 'purple';

                return (
                    <Badge ml='1' px='4' colorScheme={colorScheme} fontSize="font-base">
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
                const colorScheme = error === 0 ? "alphaWhite" : 'red';

                return (
                    <Badge ml='1' px='4' colorScheme={colorScheme} fontSize="font-base">
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

                if (!creatorData) {
                    return <div className="text-sm text-gray-500">Not available</div>;
                }

                return (
                    <div className="flex flex-row gap-2 items-center">
                        <Image
                            borderRadius='full'
                            boxSize='20px'
                            src={creatorData?.photoURL}
                            alt={`Profile of ${creatorData?.name}`}
                        />
                        <div className="text-sm flex flex-row gap-1 items-center text-gray-500">
                            {creatorData?.name}
                        </div>
                    </div>
                );
            }
        },
    ];

    return (
        <Table data={quests} columns={columns} onRowClick={onToggle} />
    )
}
