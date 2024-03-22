import { FC } from "react";
import Table from "../../components/tavern-base-ui/Table";
import { ColumnDef } from "@tanstack/react-table";
import { Image } from "@chakra-ui/react";
import Button from "../../components/tavern-base-ui/button/Button";
import { useNavigate } from "react-router-dom";

type ShellTableProps = {
    shells: any;
}
const ShellTable: FC<ShellTableProps> = ({ shells }) => {
    const nav = useNavigate();

    const columns: ColumnDef<any>[] = [
        {
            id: "id",
            header: 'Shell id',
            accessorFn: row => row?.id,
            footer: props => props.column.id,
            enableSorting: false,
            cell: (cellData: any) => {
                const data = cellData.getValue();
                return <div className="text-sm text-gray-500">{data}</div>;
            }
        },
        {
            id: "activeUsers",
            header: 'Active users',
            accessorFn: row => row?.activeUsers,
            footer: props => props.column.id,
            enableSorting: false,
            cell: (cellData: any) => {
                const users = cellData.getValue();

                if (!users || users?.length === 0) {
                    return <div className="text-sm text-gray-500">None</div>;
                }

                return (
                    <div className="flex flex-col gap-2">
                        {users.map((user: any) => (
                            <div className="flex flex-row gap-2 items-center flex-wrap">
                                <Image
                                    borderRadius='full'
                                    boxSize='20px'
                                    src={user?.photoURL}
                                    alt={`Profile of ${user?.name}`}
                                />
                                <div className="text-sm flex flex-row gap-1 items-center text-gray-500">
                                    {user?.name}
                                </div>
                            </div>
                        ))}
                    </div>
                );
            }
        },
        {
            id: "actions",
            header: 'Actions',
            accessorFn: row => row,
            footer: props => props.column.id,
            enableSorting: false,
            cell: (cellData: any) => {
                const allData = cellData.getValue();

                if (allData.closedAt) {
                    return "-";
                }
                else {
                    return <Button
                        buttonStyle={{ color: "gray", size: 'md' }}
                        buttonVariant="ghost"
                        aria-label="Join shell"
                        onClick={() => nav(`/shells/${allData.id}`)}>Join</Button>;
                }
            }
        },
    ];

    return (
        <Table data={shells} columns={columns} />
    )
}
export default ShellTable;
