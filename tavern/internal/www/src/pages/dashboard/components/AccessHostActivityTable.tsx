import React from "react";
import { ColumnDef } from "@tanstack/react-table";
import { formatDistance } from "date-fns";
import { useNavigate } from "react-router-dom";

import Table from "../../../components/tavern-base-ui/Table";
import Badge from "../../../components/tavern-base-ui/badge/Badge";

const AccessHostActivityTable = ({ hostActivity, term }: { hostActivity: any, term: string }) => {
    const currentDate = new Date();
    const navigation = useNavigate();

    const handleOnClick = (item: any) => {
        if (item?.id === "undefined") {
            return null;
        }
        navigation(`/hosts`, {
            state: [{
                'label': item?.original?.tag,
                'kind': term,
                'name': item?.original?.tag,
                'value': item?.original?.tagId
            }]
        });
    }

    const columns: ColumnDef<any>[] = [
        {
            id: "tag",
            header: term.toUpperCase(),
            accessorFn: row => row.tag,
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
                    <Badge badgeStyle={{ color: color }}>
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
                    <Badge badgeStyle={{ color: color }}>
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
        <Table columns={columns} data={hostActivity} onRowClick={handleOnClick} />
    )
}
export default AccessHostActivityTable;
