import { Badge } from "@chakra-ui/react";
import { ColumnDef } from "@tanstack/react-table";
import { formatDistance } from "date-fns";
import Table from "../../../components/tavern-base-ui/Table";
import { useNavigate } from "react-router-dom";
import { QuestProps, Task } from "../../../utils/consts";

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
    const questTableData = getInitialQuestsTableData(quests);
    const navigate = useNavigate();
    

    const currentDate = new Date();

    const onToggle = (row:any) => {
        navigate(`/quests/${row?.original?.id}`)
    }

    const columns: ColumnDef<any>[] = [
        {
            id: "name",
            header: 'Quest name',
            accessorFn: row => row.name,
            footer: props => props.column.id,
            sortingFn: "alphanumeric"
        },
        {
            id: "lastUpdated",
            header: 'Last updated',
            accessorFn: row => formatDistance(new Date(row.lastUpdated), currentDate),
            footer: props => props.column.id,
           sortingFn: (
                rowA,
                rowB,
              ) => {
                const numA = new Date(rowA?.original?.lastUpdated as string);
                const numB= new Date(rowB?.original?.lastUpdated as string);
            
                return numA < numB ? 1 : numA > numB ? -1 : 0;
              }
        },
        {
            id: "finished",
            header: 'Finished Tasks',
            accessorFn: row => row,
            cell: (row: any) => {
                const rowData = row.row.original;
                const finished = rowData.finished;
                const allTasks = rowData.inprogress + rowData.queued + rowData.finished;

                if(finished < allTasks ){
                    return (
                        <Badge ml='1' px='4' colorScheme='alphaWhite' fontSize="font-base">
                            {finished}/{allTasks}
                        </Badge>
                    );
                }

                return (
                    <Badge ml='1' px='4' colorScheme='green' fontSize="font-base">
                        {finished}/{allTasks}
                    </Badge>
                );
            },
            footer: (props:any) => props.column.id,
            enableSorting: false,
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
            sortingFn: "alphanumeric"
        },
        {
            id: "inprogress",
            header: 'In progress tasks',
            accessorKey: "inprogress",
            footer: props => props.column.id,
            sortingFn: "alphanumeric"
        },      
        {
            id: "queued",
            header: 'Queued tasks',
            accessorKey: "queued",
            footer: props => props.column.id,
            sortingFn: "alphanumeric"
        },  
    ];


    function getInitialQuestsTableData(data:any ){ 
        const formattedData = data?.map( (quest: QuestProps) => {
            console.log(quest);
            const taskDetails = quest.tasks.reduce( (map:any, task: Task)=> {
                const modMap = {...map};

                if(task.execFinishedAt){
                    modMap.finished += 1;
                }
                else if(task.execStartedAt){
                    modMap.inprogress += 1;
                }
                else{
                    modMap.queued += 1;
                }

                if(new Date(task.lastModifiedAt) > new Date(modMap.lastUpdated) ){
                    modMap.lastUpdated = task.lastModifiedAt;
                }

                if(task.output !== ""){
                    modMap.outputCount += 1;
                }

                return modMap
            },
                {
                finished: 0,
                inprogress: 0,
                queued: 0,
                outputCount: 0,
                lastUpdated: null
                }
            );

            return {
                id: quest.id,
                name: quest.name,
                ...taskDetails
            }
        });
        return formattedData.sort(function(a:any,b:any){return new Date(b.lastUpdated).getTime() - new Date(a.lastUpdated).getTime()});
    };

    return (
        <Table data={questTableData} columns={columns} onRowClick={onToggle} />
    )
}