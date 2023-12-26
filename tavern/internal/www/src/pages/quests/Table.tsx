import { Tooltip } from "@chakra-ui/react";
import { ColumnDef } from "@tanstack/react-table";

const Table = (props: any) => {
    const {outputData} = props;
    
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
        <Table data={outputData} columns={columns} onRowClick={()=>null} />
    )
}
export default Table;