import { differenceInMinutes, format, startOfHour } from "date-fns";
import { useCallback, useEffect, useState } from "react";
import { TomeTag } from "../../../utils/consts";

type UniqueCountObject = {
    [key: string] : number
}

export const useOverviewData = (data: Array<any>) => {
    const [loading, setLoading] = useState(true);
    const [formattedData, setFormattedData] = useState({
        tomeUsage: [],
        taskTimeline: [],
        taskTactics: []
    }) as Array<any>;


    const applyUniqueTermCount = useCallback((term: string, termCountObject: UniqueCountObject)=> {
        if (!(term in termCountObject)) {
            return termCountObject[term] = 1;
        }
        else {
            return termCountObject[term] += 1;
        }
    },[]);

    const getTermUsage = useCallback((termCountObject: UniqueCountObject, sortByAsc: boolean) => {
        const keys = Object.keys(termCountObject);
        const dataUsage = [];

        for (let key in keys) {
            dataUsage.push({
                name: keys[key],
                "task count": termCountObject[keys[key]]
            });
        }
        if(sortByAsc){
            dataUsage.sort((a, b) => b["task count"] -  a["task count"]);
        }
        else{
            dataUsage.sort((a, b) => a["task count"] -  b["task count"]);
        }
        return dataUsage;
    },[]);

    const modifyTaskTimeline = useCallback((task: any, tasksTimeline: Array<any>)=> {
        const created = new Date(task?.node?.createdAt);
        const tactic = task?.node?.quest?.tome?.tactic?.toLowerCase();

        if(tasksTimeline.length === 0){
            tasksTimeline.push({
                label: format(created, "iii haaa"),
                timestamp: startOfHour(created),
                ["tasks created"]: 1,
                [tactic]:1,
            })
        }
        else{
            const lastValue = tasksTimeline[tasksTimeline.length -1];

            if(differenceInMinutes(created, lastValue.timestamp) > 60){
                tasksTimeline.push({
                    label: format(created, "iii haaa"),
                    timestamp: startOfHour(created),
                    ["tasks created"]: 1,
                    [tactic]: 1,
                });

            }
            else{
                tasksTimeline[tasksTimeline.length -1]["tasks created"] += 1;

                if(tactic in tasksTimeline[tasksTimeline.length -1]){
                    tasksTimeline[tasksTimeline.length -1][tactic] += 1;
                }
                else{
                    tasksTimeline[tasksTimeline.length -1][tactic] = 1;
                }
            }
        }


    },[]);

    const modifyUniqueTactics = useCallback((task: any, uniqueTactics: any,) => {
        const tactic = task?.node?.quest?.tome?.tactic.toLowerCase();
        if(!(tactic in uniqueTactics)){
            return uniqueTactics[tactic] = true;
        }
        return
    },[]);

    const formatOverviewData = useCallback((data: Array<any>) =>{
        const uniqueTomeCount = {} as any;
        const uniqueTactics = {} as any;
        const uniqueGroup = {} as any;
        const tasksTimeline = [] as Array<any>;

        for (let index in data){
            const groupTag = data[index]?.node?.beacon?.host?.tags.find( (tag: TomeTag) => tag.kind === "group");
            applyUniqueTermCount(data[index]?.node?.quest?.tome?.name, uniqueTomeCount);
            modifyTaskTimeline(data[index], tasksTimeline);
            modifyUniqueTactics(data[index], uniqueTactics);
            applyUniqueTermCount(groupTag?.name || "Unknown", uniqueGroup);
        }

        const overviewData = {
            tomeUsage: getTermUsage(uniqueTomeCount, true),
            taskTimelime: tasksTimeline,
            taskTactics: Object.keys(uniqueTactics),
            groupUsage: getTermUsage(uniqueGroup, false)
        }
        setLoading(false);

        setFormattedData(overviewData);

    },[getTermUsage]);

    useEffect(()=> {
        formatOverviewData(data);
    },[data]);


    return {
        loading,
        formattedData
    }
}
