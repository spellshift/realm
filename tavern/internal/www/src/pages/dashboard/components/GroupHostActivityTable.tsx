import { Badge } from "@chakra-ui/react";
import { BugAntIcon } from "@heroicons/react/24/outline";
import { ColumnDef } from "@tanstack/react-table";
import { formatDistance } from "date-fns";
import { useNavigate } from "react-router-dom";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import Table from "../../../components/tavern-base-ui/Table";
import { useHostAcitvityData } from "../hook/useHostActivityData";

const GroupHostActivityTable = ({ hosts }: { hosts: Array<any> }) => {
    const { loading, hostActivity, onlineHostCount, totalHostCount } = useHostAcitvityData(hosts);
    const currentDate = new Date();
    const navigation = useNavigate();

    const handleOnClick = (item: any) => {
        navigation(`/hosts`, {
            state: [{
                'label': item?.original?.group,
                'kind': 'group',
                'name': item?.original?.group,
                'value': item?.original?.tagId
            }]
        });
    }

    const columns: ColumnDef<any>[] = [
        {
            id: "group",
            header: 'Group',
            accessorFn: row => row.group,
            footer: props => props.column.id,
            enableSorting: true,
        },
        {
            id: "hostStatus",
            header: "Active beacons",
            accessorFn: row => row,
            footer: props => props.column.id,
            enableSorting: true,
            sortingFn: (
                rowA,
                rowB,
            ) => {
                const numA = rowA?.original?.online / (rowA?.original?.total);
                const numB = rowB?.original?.online / (rowB?.original?.total);

                return numA < numB ? 1 : numA > numB ? -1 : 0;
            },
            cell: (cellData: any) => {
                const rowData = cellData.getValue();
                const color = rowData.online === 0 ? "red" : "gray";
                return (
                    <Badge ml='1' px='4' colorScheme={color} fontSize="font-base">
                        {rowData.online}/{rowData.total}
                    </Badge>
                );
            }
        },
        {
            id: "lastSeenAt",
            header: 'Last callback',
            accessorFn: row => formatDistance(new Date(row.lastSeenAt), currentDate),
            footer: props => props.column.id,
            maxSize: 100,
            sortingFn: (
                rowA,
                rowB,
            ) => {
                const numA = new Date(rowA?.original?.lastSeenAt as string);
                const numB = new Date(rowB?.original?.lastSeenAt as string);

                return numA < numB ? 1 : numA > numB ? -1 : 0;
            }
        },
    ];

    if (loading) {
        return <EmptyState type={EmptyStateType.loading} label="Formatting host data..." />
    }

    return (
        <div className=" bg-white rounded-lg shadow-lg flex flex-col w-full h-full p-4">
            <div className='flex flex-row gap-4 items-center'>
                <div className="rounded-md bg-purple-900 p-4">
                    <BugAntIcon className="text-white w-8 h-8" />
                </div>
                <div className='flex flex-col'>
                    <h2 className="text-lg font-semibold text-gray-900">Active hosts</h2>
                    <h3 className='text-lg'>
                        {onlineHostCount}/{totalHostCount}
                    </h3>
                </div>
            </div>
            <div className='h-80 overflow-y-scroll'>
                <Table columns={columns} data={hostActivity} onRowClick={handleOnClick} />
            </div>
        </div>
    )
}
export default GroupHostActivityTable;
