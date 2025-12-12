import { ColumnDef } from "@tanstack/react-table";
import { formatDistance } from "date-fns";
import { useNavigate } from 'react-router-dom';

import Table from "../../../components/tavern-base-ui/Table";
import { BeaconType } from "../../../utils/consts";
import { PrincipalAdminTypes } from "../../../utils/enums";
import { checkIfBeaconOffline } from "../../../utils/utils";
import Button from "../../../components/tavern-base-ui/button/Button";
import Badge from "../../../components/tavern-base-ui/badge/Badge";

type Props = {
    beacons: Array<BeaconType>
}
const BeaconTable = (props: Props) => {
    const { beacons } = props;
    const nav = useNavigate();
    const currentDate = new Date();
    const princialColors = Object.values(PrincipalAdminTypes);

    const columns: ColumnDef<any>[] = [
        {
            id: "name",
            header: 'Beacon',
            accessorFn: row => row?.name,
            footer: props => props.column.id,
            enableSorting: true,
        },
        {
            id: "principal",
            header: "Principal",
            accessorFn: row => row?.principal,
            footer: props => props.column.id,
            enableSorting: true,
            maxSize: 100,
            sortingFn: (
                rowA,
                rowB,
            ) => {
                const numA = rowA?.original?.principal ? (princialColors.indexOf(rowA?.original?.principal) !== -1) : 0;
                const numB = rowB?.original?.principal ? (princialColors.indexOf(rowB?.original?.principal) !== -1) : 0;

                return numA < numB ? 1 : numA > numB ? -1 : 0;
            },
            cell: (cellData: any) => {
                const principal = cellData.getValue();
                const color = princialColors.indexOf(principal) === -1 ? 'gray' : 'purple';
                return (
                    <Badge badgeStyle={{ color: color }} key={principal}>{principal}</Badge>
                );
            }
        },
        {
            id: "Status",
            header: "Status",
            accessorFn: row => row,
            footer: props => props.column.id,
            enableSorting: false,
            maxSize: 80,
            cell: (cellData: any) => {
                const beaconData = cellData.getValue();
                const beaconOffline = checkIfBeaconOffline(beaconData);
                const color = beaconOffline ? "red" : "gray";
                const text = beaconOffline ? "Offline" : "Online"
                return (
                    <Badge badgeStyle={{ color: color }} >
                        {text}
                    </Badge>
                );
            }
        },
        {
            id: "lastSeenAt",
            header: 'Last callback',
            accessorFn: row => formatDistance(new Date(row.lastSeenAt), currentDate),
            footer: props => props.column.id,
            maxSize: 120,
            sortingFn: (
                rowA,
                rowB,
            ) => {
                const numA = new Date(rowA?.original?.lastSeenAt as string);
                const numB = new Date(rowB?.original?.lastSeenAt as string);

                return numA < numB ? 1 : numA > numB ? -1 : 0;
            }
        },
        {
            id: "Create quest",
            header: "",
            accessorFn: row => row,
            footer: props => props.column.id,
            enableSorting: false,
            maxSize: 100,
            cell: (cellData: any) => {
                const beaconData = cellData.getValue();
                const isOffline = checkIfBeaconOffline(beaconData);
                const id = beaconData.id;
                return (
                    <div className="flex flex-row justify-end">
                        {!isOffline &&
                            <Button
                                buttonStyle={{ color: "gray", size: 'md' }}
                                buttonVariant="ghost"
                                onClick={() =>
                                    nav("/createQuest", {
                                        state: {
                                            step: 1,
                                            beacons: [id]
                                        }
                                    })
                                }>
                                New quest
                            </Button>
                        }
                    </div>
                )
            }

        }
    ]
    return (
        <Table columns={columns} data={beacons} />
    );
}
export default BeaconTable;
