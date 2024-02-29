import React from "react";
import { Badge } from "@chakra-ui/react";
import { ColumnDef } from "@tanstack/react-table";
import { formatDistance } from "date-fns";
import { useNavigate } from "react-router-dom";

import Table from "../../../components/tavern-base-ui/Table";

const GroupHostActivityTable = ({ hostActivity }: { hostActivity: Array<any> }) => {
    const currentDate = new Date();
    const navigation = useNavigate();

    const handleOnClick = (item: any) => {
        if (item?.id === "undefined") {
            return null;
        }
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
            header: "Hosts",
            accessorFn: row => row,
            footer: props => props.column.id,
            enableSorting: true,
            sortingFn: (
                rowA,
                rowB,
            ) => {
                const numA = rowA?.original?.hostsOnline / (rowA?.original?.hostsTotal);
                const numB = rowB?.original?.hostsOnline / (rowB?.original?.hostsTotal);

                return numA < numB ? 1 : numA > numB ? -1 : 0;
            },
            cell: (cellData: any) => {
                const rowData = cellData.getValue();
                const color = rowData.hostsOnline === 0 ? "red" : "gray";
                return (
                    <Badge px='4' colorScheme={color} fontSize="font-base">
                        {rowData.hostsOnline}/{rowData.hostsTotal}
                    </Badge>
                );
            }
        },
        {
            id: "beaconStatus",
            header: "Beacons",
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
                    <Badge px='4' colorScheme={color} fontSize="font-base">
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

    return (
        <div className="flex flex-col w-full h-full">
            <div className='flex flex-row gap-4 items-center'>
                <h2 className="text-lg">Access by group</h2>
            </div>
            <div className='h-80 overflow-y-scroll'>
                <Table columns={columns} data={hostActivity} onRowClick={handleOnClick} />
            </div>
        </div>
    )
}
export default GroupHostActivityTable;
