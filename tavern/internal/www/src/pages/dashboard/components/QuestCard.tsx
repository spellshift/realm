import { useState } from "react";

import DashboardStatistic from "./DashboardStatistic";
import TaskBarChart from "./QuestTaskBarChart";
import SingleDropdownSelector from "../../../components/tavern-base-ui/SingleDropdownSelector";
import TagBarChart from "./QuestTagBarChart";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import TomeBarChart from "./QuestTomeBarChart";
import { HostEdge } from "../../../utils/interfacesQuery";
import { QuestFormattedData } from "../types";

interface QuestCardProps {
    formattedData: QuestFormattedData;
    hosts: HostEdge[];
    loading: boolean;
}

const QuestCard = ({ formattedData, hosts, loading }: QuestCardProps) => {

    const [selectedOption, setSelectedOption] = useState({
        label: "Creation time",
        value: "creation_time"
    });

    const options = [
        {
            label: "Creation time",
            value: "creation_time"
        },
        {
            label: "Group",
            value: "group",
        },
        {
            label: "Service",
            value: "service",
        },
        {
            label: "Tome",
            value: "tome",
        }
    ];

    function getChartWrapper(selectedValue: string) {
        switch (selectedValue) {
            case "creation_time":
                return (
                    <div className='h-80 overflow-y-scroll'>
                        <TaskBarChart data={formattedData?.taskTimeline || []} taskTactics={formattedData.taskTactics} loading={loading} />
                    </div>
                );
            case "group":
                return (
                    <TagBarChart data={formattedData?.groupUsage || []} loading={loading} tagKind="group" />
                );
            case "service":
                return (
                    <TagBarChart data={formattedData?.serviceUsage || []} loading={loading} tagKind="service" />
                );
            case "tome":
                return (
                    <div className='h-80 overflow-y-scroll'>
                        <TomeBarChart data={formattedData?.tomeUsage || []} loading={loading} />
                    </div>
                );
            default:
                return <EmptyState type={EmptyStateType.error} label="Error displaying tasks" />
        }
    }

    return (
        <div className="grid col-span-1  md:grid-cols-5 gap-6  bg-white rounded-lg shadow-lg p-4">
            <h3 className="text-xl col-span-1 md:col-span-5">
                Quest statistics
            </h3>
            <div className="col-span-1 md:col-span-1 flex flex-row md:flex-col gap-4 flex-wrap">
                <DashboardStatistic label="Total quests" value={formattedData.totalQuests} loading={loading} />
                <DashboardStatistic label="Total tasks" value={formattedData.totalTasks} loading={loading} />
                <DashboardStatistic label="Total outputs" value={formattedData.totalOutput} loading={loading} />
                <DashboardStatistic label="Total errors" value={formattedData.totalErrors} loading={loading} />
            </div>
            <div className="col-span-1 md:col-span-4 flex flex-col w-full h-full gap-4">
                <div className='flex flex-row gap-2 items-center'>
                    <h2 className="text-lg">Quest tasks by</h2>
                    <SingleDropdownSelector setSelectedOption={setSelectedOption} options={options} label="accessDropdown" />
                </div>
                {getChartWrapper(selectedOption.value)}
            </div>
        </div>
    );
}
export default QuestCard;
