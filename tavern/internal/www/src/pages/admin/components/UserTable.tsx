import { ColumnDef } from "@tanstack/react-table";
import Table from "../../../components/tavern-base-ui/Table";
import UserImageAndName from "../../../components/UserImageAndName";
import { useUpdateUser } from "../hooks/userMutations";
import { UserType } from "../../../utils/consts";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import Button from "../../../components/tavern-base-ui/button/Button";


type Props = {
    data: UserType[];
    currentUser: UserType;
}
const UserTable = (props: Props) => {
    const { data, currentUser } = props;
    const { submitUpdateUser } = useUpdateUser();

    const columns: ColumnDef<any>[] = [
        {
            id: "name",
            header: 'Name',
            accessorFn: row => row,
            footer: props => props.column.id,
            maxSize: 50,
            enableSorting: true,
            cell: (cellData: any) => {
                const rowData = cellData.getValue();
                return (
                    <UserImageAndName userData={rowData} />
                );
            }
        },
        {
            id: "status",
            header: "Status",
            accessorFn: row => row,
            footer: props => props.column.id,
            maxSize: 100,
            enableSorting: false,
            cell: (cellData: any) => {
                const rowData = cellData.getValue();
                return (
                    <div className="flex flex-row flex-wrap gap-1">
                        {rowData?.isActivated && <Badge>
                            <div>Activated</div>
                        </Badge>}
                        {!rowData?.isActivated && <Badge>
                            <div>Pending</div>
                        </Badge>}
                        {rowData?.isAdmin && <Badge>
                            <div>Administrator</div>
                        </Badge>}
                    </div>
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
                const isDisabled = rowData.id === currentUser.id;
                return (
                    <div className="flex flex-row flex-wrap gap-2">
                        {!rowData?.isActivated && <Button disabled={isDisabled} buttonVariant="outline" buttonStyle={{ color: "gray", size: "sm" }} onClick={() => submitUpdateUser({ "id": rowData.id, "activated": true, "admin": false })}>
                            Activate
                        </Button>}
                        {rowData?.isActivated && <Button disabled={isDisabled} buttonVariant="outline" buttonStyle={{ color: "red", size: "sm" }} onClick={() => submitUpdateUser({ "id": rowData.id, "activated": false, "admin": false })}>
                            Deactivate
                        </Button>}
                        {rowData?.isActivated && !rowData?.isAdmin && <Button disabled={isDisabled} buttonVariant="outline" buttonStyle={{ color: "gray", size: "sm" }} onClick={() => submitUpdateUser({ "id": rowData.id, "activated": true, "admin": true })}>
                            Promote
                        </Button>}
                        {rowData?.isActivated && rowData?.isAdmin && <Button disabled={isDisabled} buttonVariant="outline" buttonStyle={{ color: "red", size: "sm" }} onClick={() => submitUpdateUser({ "id": rowData.id, "activated": true, "admin": false })}>
                            Demote
                        </Button>}
                    </div>
                );
            }
        },
    ];

    return (
        <Table data={data} columns={columns} />
    );
}
export default UserTable;
