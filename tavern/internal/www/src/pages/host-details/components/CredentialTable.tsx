import { ColumnDef } from "@tanstack/react-table";
import Table from "../../../components/tavern-base-ui/Table";
import { formatDistance } from "date-fns";
import Credential from "./Credential";


const CredentialTable = ({ data }: any) => {
    const currentDate = new Date();
    const columns: ColumnDef<any>[] = [
        {
            id: "createdAt",
            header: 'Created',
            accessorFn: row => formatDistance(new Date(row.createdAt), currentDate),
            footer: props => props.column.id,
            sortingFn: (
                rowA,
                rowB,
            ) => {
                const numA = new Date(rowA?.original?.createdAt as string);
                const numB = new Date(rowB?.original?.createdAt as string);

                return numA < numB ? 1 : numA > numB ? -1 : 0;
            }
        },
        {
            id: "principal",
            header: "principal",
            accessorFn: row => row.principal,
            footer: props => props.column.id,
            maxSize: 100,
        },
        {
            id: "kind",
            header: "Credential Kind",
            accessorFn: row => row.kind,
            footer: props => props.column.id,
            enableSorting: false,
            cell: (cellData: any) => {
                const kind = cellData.getValue();
                return (
                    <div className="flex flex-row flex-wrap gap-1">
                        {kind}
                    </div>
                );
            }
        },
        {
            id: "secret",
            header: 'Secret',
            accessorFn: row => row.secret,
            footer: props => props.column.id,
            maxSize: 250,
            enableSorting: false,
            cell: (cellData: any) => {
                const secret = cellData.getValue();
                return (
                    <div className="flex justify-between">
                        <Credential secret={secret} />
                    </div>
                );
            }
        },
    ];

    return (
        <div>
            <Table data={data} columns={columns} />
        </div>
    );
}
export default CredentialTable;
