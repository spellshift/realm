import { Badge } from "@chakra-ui/react";
import { ColumnDef } from "@tanstack/react-table";
import { formatDistance } from "date-fns";
import Table from "../../../components/tavern-base-ui/Table";
import { BeaconType } from "../../../utils/consts";
import { PrincipalAdminTypes } from "../../../utils/enums";
import { checkIfBeaconOnline } from "../../../utils/utils";

type Props = {
    beacons: Array<BeaconType>
}
const BeaconTable = (props: Props) => {
    const { beacons } = props;
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
                    <div className="flex flex-row flex-wrap gap-1">
                        <Badge textTransform="none" colorScheme={color} key={principal}>{principal}</Badge>
                    </div>
                );
            }
        },
        {
            id: "Status",
            header: "Status",
            accessorFn: row => row,
            footer: props => props.column.id,
            enableSorting: false,
            cell: (cellData: any) => {
                const beaconData = cellData.getValue();
                const status = checkIfBeaconOnline(beaconData);
                const color = status === false ? "red" : "gray";
                const text = status === false ? "Offline" : "Online"
                return (
                    <Badge colorScheme={color} >
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
    ]
    return (
        <Table columns={columns} data={beacons} />
    );
}
export default BeaconTable;
