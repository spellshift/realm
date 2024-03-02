import { Badge, Image } from "@chakra-ui/react";
import { ColumnDef } from "@tanstack/react-table";
import { formatDistance } from "date-fns";
import Table from "../../../components/tavern-base-ui/Table";
import { useNavigate } from "react-router-dom";
import { QuestTableRowType } from "../../../utils/consts";


type Props = {
    quests: Array<QuestTableRowType>;
}
export const QuestTable = (props: Props) => {
    const { quests } = props;
    const navigate = useNavigate();


    const currentDate = new Date();

    const onToggle = (row: any) => {
        navigate(`/tasks/${row?.original?.id}`)
    }

    const columns: ColumnDef<any>[] = [
        {
            id: "name",
            header: 'Quest details',
            accessorFn: row => row,
            footer: props => props.column.id,
            enableSorting: false,
            minSize: 200,
            cell: (cellData: any) => {
                const questData = cellData.getValue();
                return (
                    <div className="flex flex-col">
                        <div>{questData.name}</div>
                        <div className="text-sm flex flex-row gap-1 items-center text-gray-500">
                            {questData?.tome}
                        </div>
                    </div>
                );
            },
        },
        {
            id: "lastUpdated",
            header: 'Updated',
            maxSize: 100,
            accessorFn: row => formatDistance(new Date(row.lastUpdated), currentDate),
            footer: props => props.column.id,
            sortingFn: (
                rowA,
                rowB,
            ) => {
                const numA = new Date(rowA?.original?.lastUpdated as string);
                const numB = new Date(rowB?.original?.lastUpdated as string);

                return numA < numB ? 1 : numA > numB ? -1 : 0;
            },
            enableSorting: false,
        },
        {
            id: "finished",
            header: 'Finished',
            accessorFn: row => row,
            maxSize: 60,
            cell: (row: any) => {
                const rowData = row.row.original;
                const finished = rowData.finished;
                const allTasks = rowData.inprogress + rowData.queued + rowData.finished;

                if (finished < allTasks) {
                    return (
                        <Badge ml='1' px='4' colorScheme='alphaWhite' fontSize="font-base">
                            {finished}/{allTasks}
                        </Badge>
                    );
                }

                return (
                    <Badge ml='1' px='4' colorScheme='green' fontSize="font-base">
                        {finished}/{allTasks}
                    </Badge>
                );
            },
            footer: (props: any) => props.column.id,
            enableSorting: false,
        },
        {
            id: "output",
            header: 'Output',
            accessorKey: "outputCount",
            maxSize: 60,
            cell: (cellData: any) => {
                const output = cellData.getValue();

                if (output === 0) {
                    return (
                        <Badge ml='1' px='4' colorScheme='alphaWhite' fontSize="font-base">
                            {output}
                        </Badge>
                    );
                }

                return (
                    <Badge ml='1' px='4' colorScheme='purple' fontSize="font-base">
                        {output}
                    </Badge>
                );
            },
            footer: (props: any) => props.column.id,
            enableSorting: false,
        },
        {
            id: "error",
            header: 'Error',
            accessorKey: "errorCount",
            maxSize: 60,
            cell: (cellData: any) => {
                const error = cellData.getValue();

                if (error === 0) {
                    return (
                        <Badge ml='1' px='4' colorScheme='alphaWhite' fontSize="font-base">
                            {error}
                        </Badge>
                    );
                }

                return (
                    <Badge ml='1' px='4' colorScheme='red' fontSize="font-base">
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
            accessorFn: row => row.creator,
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
