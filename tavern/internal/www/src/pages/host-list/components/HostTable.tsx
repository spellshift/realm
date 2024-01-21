import { Badge } from "@chakra-ui/react";
import { ColumnDef } from "@tanstack/react-table";
import { formatDistance } from "date-fns";
import { useMemo } from "react";
import { useNavigate } from "react-router-dom";
import Table from "../../../components/tavern-base-ui/Table";


const HostTable = ({ data }: any) => {
    const currentDate = new Date();
    const navigate = useNavigate();

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
                    <div className="flex flex-col">
                        <div>{rowData.name}</div>
                        <div className="flex flex-row flex-wrap gap-1">
                            {rowData?.tags.map((tag: any) => {
                                return <Badge key={tag.id}>{tag.name}</Badge>
                            })}
                            <Badge>{rowData?.primaryIP}</Badge>
                            <Badge>{rowData?.platform}</Badge>
                        </div>
                    </div>
                );
            }
        },
        {
            id: "beaconStatus",
            header: "Active beacons",
            accessorFn: row => row.beaconStatus,
            footer: props => props.column.id,
            cell: (cellData: any) => {
                const beacons = cellData.getValue();
                const color = beacons.online === 0 ? "red" : "gray";
                return (
                    <Badge ml='1' px='4' colorScheme={color} fontSize="font-base">
                        {beacons.online}/{beacons.offline + beacons.online}
                    </Badge>
                );
            }
        },
        {
            id: "lastSeenAt",
            header: 'Last callback',
            accessorFn: row => formatDistance(new Date(row.lastSeenAt), currentDate),
            footer: props => props.column.id,
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
