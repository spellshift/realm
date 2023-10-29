import { useQuery } from "@apollo/client";
import { GET_QUEST_QUERY } from "../../utils/queries";
import { QuestProps, Task, OutputTableProps } from "../../utils/consts";
import { useCallback, useContext, useEffect, useState } from "react";
import { getFilterBarSearchTypes } from "../../components/utils/utils";
import { AuthorizationContext } from "../../context/AuthorizationContext";

export const useOutputResult = () : {
    loading:boolean, 
    tableData: Array<OutputTableProps>, 
    filteredData: Array<OutputTableProps>, 
    setSearch: (arg:string) => void,
    setTypeFilters: (arg:any) => void,
    setShowOnlyMyQuests: (arg:any) => void
} => {
    const {data: userData} = useContext(AuthorizationContext);
    const { loading, data } = useQuery(GET_QUEST_QUERY);
    const [tableData, setTableData] = useState<Array<OutputTableProps>>([])
    const [filteredData, setFilteredData] = useState<Array<OutputTableProps>>([]);
    const [search, setSearch] = useState("");
    const [typeFilters, setTypeFilters] = useState([]);
    const [showOnlyMyQuests, setShowOnlyMyQuests] = useState(false); 

    const getAllOutputs = useCallback((questData: Array<QuestProps>) => {
        const output = [] as Array<OutputTableProps>;

        questData?.forEach((quest:QuestProps)=> {
            quest?.tasks?.forEach((task:Task)=> {

                if(task.output && task.output !== ""){
                    output.push({
                        quest: quest.name,
                        creator: quest.creator,
                        tome: quest.tome.name,
                        beacon: task.beacon.name,
                        service: task.beacon?.host?.tags.find( (obj : any) => {
                            return obj?.kind === "service"
                        })?.name || null,
                        group: task.beacon?.host?.tags.find( (obj : any) => {
                            return obj?.kind === "group"
                        })?.name || null,
                        output: task.output,
                        taskDetails: task
                    });
                }
            });
        });

        return output;
    },[]);

    const filterByTypes = useCallback((inData: Array<OutputTableProps>, typeFilters: any) => {
        if(typeFilters.length < 1){
            return inData;
        }

        const searchTypes = getFilterBarSearchTypes(typeFilters);

        return inData.filter( (data) => {
            let match = true;

            if(searchTypes.beacon.length > 0){
                // If a beacon filter is applied ignore other filters to just match the beacon
                if(searchTypes.beacon.indexOf(data.beacon) > -1){
                    match = true;
                }
                else{
                    return false;
                }
            }

            if(searchTypes.service.length > 0){
                if(data.service && searchTypes.service.indexOf(data.service) > -1){
                    match = true;
                }
                else{
                    return false;
                }
            }

            if(searchTypes.group.length > 0){
                if(data.group && searchTypes.group.indexOf(data.group) > -1){
                    match = true;
                }
                else{
                    return false;
                }
            }

            return match;
        });
    },[]);

    useEffect(()=>{
        if(!loading && data){
            setTableData(getAllOutputs(data?.quests));
        }
    },[loading,data, setTableData]);

    useEffect(()=> {
        if(tableData.length === 0){
            return
        };

        let result = tableData.filter(item => item?.output?.toLowerCase().includes(search.toLowerCase()));
        
        if(showOnlyMyQuests){
            result = result.filter(item => item?.creator?.id === userData?.me?.id);
        }

        result = filterByTypes(result, typeFilters);
        setFilteredData(result);
    },[tableData, search, typeFilters, setFilteredData, showOnlyMyQuests, userData]);


    return {
        loading,
        tableData,
        filteredData,
        setSearch,
        setTypeFilters,
        setShowOnlyMyQuests
    };
}