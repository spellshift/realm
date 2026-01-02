import { ColumnDef } from "@tanstack/react-table";
import { formatDistance } from "date-fns";
import { useNavigate } from 'react-router-dom';

import Table from "../../../components/tavern-base-ui/Table";
import { PrincipalAdminTypes, SupportedTransports } from "../../../utils/enums";
import { checkIfBeaconOffline, getEnumKey } from "../../../utils/utils";
import Button from "../../../components/tavern-base-ui/button/Button";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import { BeaconEdge } from "../../../utils/interfacesQuery";

const BeaconTable = ({ beacons }: { beacons: Array<BeaconEdge> }) => {
    console.log(beacons);
    const nav = useNavigate();
    const currentDate = new Date();
    const princialColors = Object.values(PrincipalAdminTypes);

    const columns: ColumnDef<any>[] = [
        {
            id: "name",
            header: 'Beacon',
            accessorFn: (row: BeaconEdge) => row?.node?.name,
            footer: props => props.column.id,
            enableSorting: false,
        },
        {
            id: "principal",
            header: "Principal",
            accessorFn: (row: BeaconEdge) => row?.node?.principal,
            footer: props => props.column.id,
            enableSorting: false,
            maxSize: 100,
            cell: (cellData: any) => {
                const principal = cellData.getValue();
                const color = princialColors.indexOf(principal) === -1 ? 'gray' : 'purple';
                return (
                    <Badge badgeStyle={{ color: color }} key={principal}>{principal}</Badge>
                );
            }
        },
        {
            id: "transport",
            header: "Transport",
            accessorFn: (row: BeaconEdge) => row?.node?.transport,
            footer: props => props.column.id,
            enableSorting: false,
            maxSize: 80,
            cell: (cellData: any) => {
                const transport = cellData.getValue();
                if (!transport) {
                    return null;
                }
                return (
                    <Badge>{getEnumKey(SupportedTransports, transport)}</Badge>
                );
            }
        },
        {
            id: "Status",
            header: "Status",
            accessorFn: (row: BeaconEdge) => row.node,
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
            accessorFn: (row: BeaconEdge) => formatDistance(new Date(row?.node.lastSeenAt), currentDate),
            footer: props => props.column.id,
            maxSize: 120,
            enableSorting: false
        },
        {
            id: "Create quest",
            header: "",
            accessorFn: (row: BeaconEdge) => row.node,
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
