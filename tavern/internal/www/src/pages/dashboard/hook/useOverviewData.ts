import { differenceInMinutes, format, startOfHour } from "date-fns";
import { useCallback, useEffect, useState } from "react";

import { TomeTag } from "../../../utils/consts";

type UniqueCountObject = {
    [key: string] : {
        name: string,
        value: number,
        id: any
    }
}

export const useOverviewData = (data: Array<any>) => {
    const [loading, setLoading] = useState(false);
    const [formattedData, setFormattedData] = useState({
        tomeUsage: [],
        taskTimeline: [],
        taskTactics: [],
        groupUsage: [],
        totalQuests: 0,
        totalOutput: 0,
        totalTasks: 0
    }) as Array<any>;


    const applyUniqueTermCount = useCallback((term: string, id: any, termCountObject: UniqueCountObject)=> {
        if (!(term in termCountObject)) {
            return termCountObject[term] = {
                name: term,
                value: 1,
                id: id
            };
        }
        else {
            return termCountObject[term].value += 1;
        }
    },[]);

    const getTermUsage = useCallback((termCountObject: UniqueCountObject, sortByAsc: boolean) => {
        const keys = Object.keys(termCountObject);
        const dataUsage = [];

        for (let key in keys) {
            dataUsage.push({
                name: keys[key],
                "task count": termCountObject[keys[key]].value,
                "id": termCountObject[keys[key]].id
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
                "tasks created": 1,
                [tactic]:1,
            })
        }
        else{
            const lastValue = tasksTimeline[tasksTimeline.length -1];

            if(differenceInMinutes(created, lastValue.timestamp) > 60){
                tasksTimeline.push({
                    label: format(created, "iii haaa"),
                    timestamp: startOfHour(created),
                    "tasks created": 1,
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

    const modifyTaskWithOutput = useCallback((task: any, tasksWithOutput: number) => {
        const hasOuput = task?.node?.outputSize > 0;
        if(hasOuput){
            return tasksWithOutput += 1;
        }
        else{
            return tasksWithOutput
        }
    },[]);

    const formatOverviewData = useCallback((data: Array<any>) =>{
        const uniqueQuestCount = {} as any;
        const uniqueTomeCount = {} as any;
        const uniqueTactics = {} as any;
        const uniqueGroup = {} as any;
        const tasksTimeline = [] as Array<any>;
        let tasksWithOutput = 0;

        for (let index in data){
            const groupTag = data[index]?.node?.beacon?.host?.tags.find( (tag: TomeTag) => tag.kind === "group");
            applyUniqueTermCount(data[index]?.node?.quest?.id, data[index]?.node?.quest?.id, uniqueQuestCount);
            applyUniqueTermCount(groupTag?.name, groupTag?.id, uniqueGroup);
            applyUniqueTermCount(data[index]?.node?.quest?.tome?.name, data[index]?.node?.quest?.tome.id, uniqueTomeCount);
            modifyTaskTimeline(data[index], tasksTimeline);
            modifyUniqueTactics(data[index], uniqueTactics);
            tasksWithOutput = modifyTaskWithOutput(data[index], tasksWithOutput);
        }

        const overviewData = {
            tomeUsage: getTermUsage(uniqueTomeCount, true),
            taskTimelime: tasksTimeline,
            taskTactics: Object.keys(uniqueTactics),
            groupUsage: getTermUsage(uniqueGroup, false),
            totalQuests: Object.keys(uniqueQuestCount).length,
            totalOutput: tasksWithOutput,
            totalTasks: data.length
        }
        setLoading(false);

        setFormattedData(overviewData);

    },[getTermUsage, applyUniqueTermCount, modifyTaskTimeline, modifyTaskWithOutput, modifyUniqueTactics]);

    useEffect(()=> {
        formatOverviewData(data);
    },[data, formatOverviewData]);


    return {
        loading,
        formattedData
    }
}
