import { Badge, Image, Button } from "@chakra-ui/react";
import { ColumnDef } from "@tanstack/react-table";
import { formatDistance } from "date-fns";
import { useNavigate } from "react-router-dom";
import Table from "../../../components/tavern-base-ui/Table";
import { PrincipalAdminTypes } from "../../../utils/enums";
import { CheckCircleIcon, ShieldCheckIcon, XCircleIcon } from "@heroicons/react/24/outline";
import UserImageAndName from "../../../components/UserImageAndName";


const UserTable = ({ data }: any) => {
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
                    <UserImageAndName userData={rowData}/>
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
                            <div className="flex flex-row gap-1 justify-center items-center p-1">
                                <div>Activated</div>
                            </div>
                        </Badge>}
                        {!rowData?.isActivated && <Badge>
                            <div className="flex flex-row gap-1 justify-center items-center p-1" >
                                <div>Pending</div>
                            </div>
                        </Badge>}
                        {rowData?.isAdmin && <Badge>
                            <div className="flex flex-row gap-1 justify-center items-center p-1" >
                                <div>Administrator</div>
                            </div>
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
                return (
                    <div className="flex flex-row flex-wrap gap-1">
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
export default UserTable;
