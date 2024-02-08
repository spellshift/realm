import { Badge, Image, Button } from "@chakra-ui/react";
import { ColumnDef } from "@tanstack/react-table";
import { formatDistance } from "date-fns";
import { useNavigate } from "react-router-dom";
import UserTile from "../../../components/UserTile";
import Table from "../../../components/tavern-base-ui/Table";
import { PrincipalAdminTypes } from "../../../utils/enums";


const UserTable = ({ data }: any) => {
    const currentDate = new Date();
    const navigate = useNavigate();
    const princialColors = Object.values(PrincipalAdminTypes);
    const onToggle = (row: any) => {
        navigate(`/users/${row?.id}`)
    }

    const columns: ColumnDef<any>[] = [
        {
            id: "photo",
            header: '',
            accessorFn: row => row,
            footer: props => props.column.id,
            maxSize: 50,
            enableSorting: false,
            cell: (cellData: any) => {
                const rowData = cellData.getValue();
                return (
                    <Image
                        borderRadius='full'
                        boxSize='40px'
                        src={rowData?.photoURL}
                        alt={`Profile of ${rowData?.name}`}
                    />
                );
            }
        },
        {
            id: "name",
            header: "Name",
            accessorFn: row => row,
            footer: props => props.column.id,
            maxSize: 100,
            enableSorting: false,
            cell: (cellData: any) => {
                const rowData = cellData.getValue();
                return (
                    <UserTile data={rowData} />
                );
            }
        },
        {
            id: "actions",
            header: "Actions",
            accessorFn: row => row,
            footer: props => props.column.id,
            maxSize: 100,
            enableSorting: false,
            cell: (cellData: any) => {
                const rowData = cellData.getValue();
                return (
                    <>
                      {!rowData?.isActivated && <Button size={"sm"}>
                        Activate
                      </Button>}
                      {rowData?.isActivated && <Button size={"sm"}>
                        Deactivate
                      </Button>}
                      {rowData?.isActivated && !rowData?.isAdmin && <Button size={"sm"}>
                        Promote
                      </Button>}
                      {rowData?.isActivated && rowData?.isAdmin && <Button size={"sm"}>
                        Demote
                      </Button>}
                    </>
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
export default UserTable;
