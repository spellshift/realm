import { Badge, Tooltip } from "@chakra-ui/react";
import { ColumnDef } from "@tanstack/react-table";
import { formatDistance } from "date-fns";
import Table from "../../../components/tavern-base-ui/Table";
import { useNavigate } from "react-router-dom";
import { OutputTableProps, QuestProps, Task } from "../../../utils/consts";


type Props = {
    outputData: Array<OutputTableProps>;
    onToggle: (e:any ) => void;
}
export const OutputTable = (props: Props) => {
    const {outputData, onToggle} = props;
    const columns: ColumnDef<any>[] = [
        {
            id: "quest",
            header: 'Quest name',
            accessorFn: row => row.quest,
            footer: props => props.column.id,
            sortingFn: "alphanumeric"
        },
        {
            id: "tome",
            header: 'Tome name',
            accessorFn: row => row.tome,
            footer: props => props.column.id,
            sortingFn: "alphanumeric"
        },
        {
            id: "beacon",
            header: 'Beacon name',
            accessorFn: row => row.beacon,
            footer: props => props.column.id,
            sortingFn: "alphanumeric"
        },
        {
            id: "service",
            header: 'Service name',
            accessorFn: row => row.service,
            footer: props => props.column.id,
            sortingFn: "alphanumeric"
        },
        {
            id: "group",
            header: 'Group name',
            accessorFn: row => row.group,
            footer: props => props.column.id,
            sortingFn: "alphanumeric"
        },
        {
            accessorKey: 'output',
            header: 'output',
            cell: (cellData: any) => {
                const output = cellData.getValue();
                return (
                    <Tooltip label={output.length > 500 ? "Click to see output" : output} aria-label='Task output'>
                        <div>
                        {output?.substring(0,50)}{output.length > 50 && "..."}
                        </div>
                    </Tooltip>
                );
            },
            footer: props => props.column.id,
        },
    ];

    return (
        <Table data={outputData} columns={columns} onRowClick={onToggle} />
    )
}