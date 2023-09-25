import { Badge } from "@chakra-ui/react";
import { ColumnDef } from "@tanstack/react-table";
import { formatDistance } from "date-fns";
import Table from "../../../components/tavern-base-ui/Table";
import { useNavigate } from "react-router-dom";

type QuestTableProps = {
    id: string,
    name: string,
    finished: number,
    inprogress: number,
    queued: number,
    outputCount: number,
    lastUpdated: string | null,
}

type Props = {
    quests: Array<QuestTableProps>;
}
export const QuestTable = (props: Props) => {
    const {quests} = props;
    const navigate = useNavigate();
    

    const currentDate = new Date();

    const onToggle = (row:any) => {
        navigate(`/quests/${row?.original?.id}`)
    }

    const columns: ColumnDef<any>[] = [
        {
            id: "quest",
            header: 'Quest name',
            accessorFn: row => row.name,
            footer: props => props.column.id,
        },
        {
            id: "updated",
            header: 'Last updated',
            accessorFn: row => formatDistance(new Date(row.lastUpdated), currentDate),
            footer: props => props.column.id,
        },
        {
            id: "finished",
            header: 'Finished Tasks',
            accessorKey: "finished",
            cell: (cellData: any) => {
                const finished = cellData.getValue();

                if(finished === 0 ){
                    return (
                        <Badge ml='1' px='4' colorScheme='alphaWhite' fontSize="font-base">
                            {finished}
                        </Badge>
                    );
                }

                return (
                    <Badge ml='1' px='4' colorScheme='green' fontSize="font-base">
                        {finished}
                    </Badge>
                );
            },
            footer: (props:any) => props.column.id,
        },
        {
            id: "output",
            header: 'Output available',
            accessorKey: "outputCount",
            cell: (cellData: any) => {
                const output = cellData.getValue();

                if(output === 0 ){
                    return (
                        <Badge ml='1' px='4' colorScheme='alphaWhite' fontSize="font-base">
                            {output}
                        </Badge>
                    );
                }

                return (
                    <Badge ml='1' px='4' colorScheme='purple' fontSize="font-base">
                        {output}
                    </Badge>
                );
            },
            footer: (props:any) => props.column.id,
        },
        {
            id: "inprogress",
            header: 'In progress tasks',
            accessorKey: "inprogress",
            footer: props => props.column.id,
        },      
        {
            id: "queued",
            header: 'Queued tasks',
            accessorKey: "queued",
            footer: props => props.column.id,
        },  
        // {
        //     accessorKey: 'output',
        //     header: 'output',
        //     cell: (cellData: any) => {
        //         const output = cellData.getValue();
        //         return (
        //             <Tooltip label={output.length > 500 ? "Click to see output" : output} aria-label='Task output'>
        //                 <div>
        //                 {output?.substring(0,50)}{output.length > 50 && "..."}
        //                 </div>
        //             </Tooltip>
        //         );
        //     },
        //     footer: props => props.column.id,
        // },
];

    return (
        <Table data={quests} columns={columns} onRowClick={onToggle} />
    )
}