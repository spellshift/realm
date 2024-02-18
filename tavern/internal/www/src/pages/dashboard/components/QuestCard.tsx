import React from "react";

import DashboardStatistic from "./DashboardStatistic";
import TaskBarChart from "./TaskBarChart";

const QuestCard = ({ formattedData, loading }: { formattedData: any, loading: boolean }) => {

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
            <div className="col-span-1 md:col-span-4">
                <TaskBarChart data={formattedData?.taskTimelime || []} taskTactics={formattedData.taskTactics} loading={loading} />
            </div>
        </div>
    );
}
export default QuestCard;
