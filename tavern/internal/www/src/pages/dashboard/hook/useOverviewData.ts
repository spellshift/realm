import { differenceInMinutes, format, startOfHour } from "date-fns";
import { useCallback, useEffect, useState } from "react";

import { TomeTag } from "../../../utils/consts";
import { TaskChartKeys } from "../../../utils/enums";

type UniqueTaskCountObject = {
    [key: string] : {
        name: string;
        tasksError: number;
        tasksNoError: number;
        id: any;
    }
}
export const useOverviewData = (data: any) => {
    const [loading, setLoading] = useState(false);
    const [formattedData, setFormattedData] = useState({
        tomeUsage: [],
        taskTimeline: [],
        taskTactics: [],
        groupUsage: [],
        serviceUsage: [],
        platformUsage: [],
        totalQuests: 0,
        totalOutput: 0,
        totalTasks: 0,
        totalErrors: 0
    }) as any;


    const applyUniqueTermCount = useCallback((term: string, id: any, hasError: boolean, termCountObject: UniqueTaskCountObject)=> {
        if (!(term in termCountObject)) {
            return termCountObject[term] = {
                name: term,
                tasksNoError: hasError ? 0 : 1,
                tasksError: hasError ? 1 : 0,
                id: id
            };
        }
        else {
            if(hasError){
                return termCountObject[term].tasksError += 1
            }
            else{
                return termCountObject[term].tasksNoError += 1
            }
        }
    },[]);

    const getTermUsage = useCallback((termCountObject: UniqueTaskCountObject, sortByAsc: boolean) => {
        const keys = Object.keys(termCountObject);
        const dataUsage = [];

        for (let key in keys) {
            dataUsage.push({
                name: keys[key],
                [TaskChartKeys.taskError]: termCountObject[keys[key]].tasksError,
                [TaskChartKeys.taskNoError]: termCountObject[keys[key]].tasksNoError,
                "id": termCountObject[keys[key]].id
            });
        }
        if(sortByAsc){
            dataUsage.sort((a, b) => b[TaskChartKeys.taskNoError] -  a[TaskChartKeys.taskNoError]);
        }
        else{
            dataUsage.sort((a, b) => a[TaskChartKeys.taskNoError] -  b[TaskChartKeys.taskNoError]);
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
                [TaskChartKeys.taskCreated]: 1,
                [tactic]:1,
            })
        }
        else{
            const lastValue = tasksTimeline[tasksTimeline.length -1];

            if(differenceInMinutes(created, lastValue.timestamp) > 60){
                tasksTimeline.push({
                    label: format(created, "iii haaa"),
                    timestamp: startOfHour(created),
                    [TaskChartKeys.taskCreated]: 1,
                    [tactic]: 1,
                });

            }
            else{
                tasksTimeline[tasksTimeline.length -1][TaskChartKeys.taskCreated] += 1;

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
        const uniqueQuestCount = {} as any;
        const uniqueTomeCount = {} as any;
        const uniqueTactics = {} as any;
        const uniqueGroup = {} as any;
        const uniqueService = {} as any;
        const tasksTimeline = [] as Array<any>;
        let tasksWithOutput = 0;
        let tasksWithError = 0;

        for (let index in data){
            const groupTag = data[index]?.node?.beacon?.host?.tags.find( (tag: TomeTag) => tag.kind === "group");
            const serviceTag = data[index]?.node?.beacon?.host?.tags.find( (tag: TomeTag) => tag.kind === "service");
            applyUniqueTermCount(data[index]?.node?.quest?.id, data[index]?.node?.quest?.id, false, uniqueQuestCount);
            applyUniqueTermCount(groupTag?.name, groupTag?.id, data[index]?.node?.error.length > 0, uniqueGroup);
            applyUniqueTermCount(serviceTag?.name, serviceTag?.id, data[index]?.node?.error.length > 0, uniqueService);
            applyUniqueTermCount(data[index]?.node?.quest?.tome?.name, data[index]?.node?.quest?.tome.id, data[index]?.node?.error.length > 0, uniqueTomeCount);
            modifyTaskTimeline(data[index], tasksTimeline);
            modifyUniqueTactics(data[index], uniqueTactics);

            if (data[index]?.node?.outputSize > 0) tasksWithOutput += 1;
            if (data[index]?.node?.error?.length > 0) tasksWithError += 1;

        }

        const overviewData = {
            tomeUsage: getTermUsage(uniqueTomeCount, true),
            taskTimelime: tasksTimeline,
            taskTactics: Object.keys(uniqueTactics),
            groupUsage: getTermUsage(uniqueGroup, false),
            serviceUsage: getTermUsage(uniqueService, false),
            totalQuests: Object.keys(uniqueQuestCount).length,
            totalOutput: tasksWithOutput,
            totalTasks: data.length,
            totalErrors: tasksWithError
        }
        setLoading(false);

        setFormattedData(overviewData);

    },[getTermUsage, applyUniqueTermCount, modifyTaskTimeline, modifyUniqueTactics]);

    useEffect(()=> {
        if(data && data?.tasks?.edges.length > 0){
            formatOverviewData(data?.tasks?.edges);
        }
    },[data, formatOverviewData]);


    return {
        loading,
        formattedData
    }
}
