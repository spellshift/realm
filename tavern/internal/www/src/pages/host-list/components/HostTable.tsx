import { ColumnDef } from "@tanstack/react-table";
import { useNavigate } from "react-router-dom";
import HostTile from "../../../components/HostTile";
import Table from "../../../components/tavern-base-ui/Table";
import { PrincipalAdminTypes } from "../../../utils/enums";
import Badge from "../../../components/tavern-base-ui/badge/Badge";


const HostTable = ({ data }: any) => {
    const navigate = useNavigate();
    const princialColors = Object.values(PrincipalAdminTypes);
    const onToggle = (row: any) => {
        navigate(`/hosts/${row?.original?.id}`)
    }

    const columns: ColumnDef<any>[] = [
        {
            id: "name",
            header: 'Host details',
            accessorFn: row => row,
            footer: props => props.column.id,
            enableSorting: false,
            cell: (cellData: any) => {
                const rowData = cellData.getValue();
                return (
                    <HostTile data={rowData} />
                );
            }
        },
        {
            id: "beaconStatus",
            header: "Online beacons",
            accessorFn: row => row.beaconStatus,
            footer: props => props.column.id,
            maxSize: 100,
            sortingFn: (
                rowA,
                rowB,
            ) => {
                const numA = rowA?.original?.beaconStatus.online / (rowA?.original?.beaconStatus.offline + rowA?.original?.beaconStatus.online);
                const numB = rowB?.original?.beaconStatus.online / (rowB?.original?.beaconStatus.offline + rowB?.original?.beaconStatus.online);

                return numA < numB ? 1 : numA > numB ? -1 : 0;
            },
            cell: (cellData: any) => {
                const beacons = cellData.getValue();
                const color = beacons.online === 0 ? "red" : "gray";
                return (
                    <Badge badgeStyle={{ color: color }}>
                        {beacons.online}/{beacons.offline + beacons.online}
                    </Badge>
                );
            }
        },
        {
            id: "beaconPrincipals",
            header: "Beacon principals",
            accessorFn: row => row.beaconPrincipals,
            footer: props => props.column.id,
            sortingFn: (
                rowA,
                rowB,
            ) => {
                const numA = rowA?.original?.beaconPrincipals ? rowA?.original?.beaconPrincipals.filter((princial: any) => princialColors.indexOf(princial) !== -1).length : 0;
                const numB = rowA?.original?.beaconPrincipals ? rowB?.original?.beaconPrincipals.filter((princial: any) => princialColors.indexOf(princial) !== -1).length : 0;

                return numA < numB ? 1 : numA > numB ? -1 : 0;
            },
            cell: (cellData: any) => {
                const beaconPrincipals = cellData.getValue();
                return (
                    <div className="flex flex-row flex-wrap gap-1">

                        {beaconPrincipals.map((principal: any) => {
                            const color = princialColors.indexOf(principal) === -1 ? 'gray' : 'purple';
                            return <Badge badgeStyle={{ color: color }} key={principal}>{principal}</Badge>
                        })}
                    </div>
                );
            }
        },
        {
            id: "lastSeenAt",
            header: 'Last callback',
            accessorFn: row => row.formattedLastSeenAt,
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
        <div>
            <Table data={data} columns={columns} onRowClick={onToggle} />
        </div>
    );
}
export default HostTable;
