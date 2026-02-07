import { ColumnDef } from "@tanstack/react-table";
import { useNavigate } from "react-router-dom";
import HostTile from "../../components/HostTile";
import Table from "../../components/tavern-base-ui/table/Table";
import { PrincipalAdminTypes } from "../../utils/enums";
import Badge from "../../components/tavern-base-ui/badge/Badge";
import { HostEdge } from "../../utils/interfacesQuery";
import { getFormatForPrincipal, getOfflineOnlineStatus } from "../../utils/utils";
import { formatDistance } from "date-fns";


const HostTable = ({ data }: { data: HostEdge[] }) => {
    const currentDate = new Date();
    const navigate = useNavigate();
    const princialColors = Object.values(PrincipalAdminTypes);
    const onToggle = (row: { original: HostEdge }) => {
        navigate(`/hosts/${row?.original?.node.id}`)
    }

    const columns: ColumnDef<any>[] = [
        {
            id: "name",
            header: 'Host details',
            accessorFn: (row: HostEdge) => row.node,
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
            accessorFn: (row: HostEdge) => row.node.beacons?.edges,
            footer: props => props.column.id,
            maxSize: 100,
            enableSorting: false,
            cell: (cellData: any) => {
                const beacons = cellData.getValue();
                const { online, offline } = getOfflineOnlineStatus(beacons);

                const color = online === 0 ? "red" : "gray";
                return (
                    <Badge badgeStyle={{ color: color }}>
                        {online}/{offline + online}
                    </Badge>
                );
            }
        },
        {
            id: "beaconPrincipals",
            header: "Beacon principals",
            accessorFn: (row: HostEdge) => row.node.beacons?.edges,
            footer: props => props.column.id,
            enableSorting: false,
            cell: (cellData: any) => {
                const beacons = cellData.getValue();
                const beaconPrincipals = getFormatForPrincipal(beacons);
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
            accessorFn: (row: HostEdge) => row.node.lastSeenAt,
            footer: props => props.column.id,
            maxSize: 100,
            enableSorting: false,
            cell: (cellData: any) => {
                const lastSeenAt = cellData.getValue();
                const formattedLastSeen = lastSeenAt ? formatDistance(new Date(lastSeenAt), currentDate) : "N/A"
                return (
                    <>{formattedLastSeen}</>
                );
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
