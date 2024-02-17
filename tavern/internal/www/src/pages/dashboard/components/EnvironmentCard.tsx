import React from "react";
import DashboardStatistic from "./DashboardStatistic";

import GroupBarChart from "./GroupBarChart";
import TomeBarChart from "./TomeBarChart";

const EnvironmentCard = ({ formattedData, hosts, loading }: { formattedData: any, hosts: Array<any>, loading: boolean }) => {

    return (
        <div className="grid grid-cols-1 md:grid-cols-5 gap-6 bg-white rounded-lg shadow-lg p-4">
            <h3 className="text-xl col-span-1 md:col-span-5">
                Engagement breakdown
            </h3>
            <div className="col-span-1 flex flex-row md:flex-col gap-4 flex-wrap">
                <DashboardStatistic label="Total groups" value={formattedData?.groupUsage?.length} loading={loading} />
                <DashboardStatistic label="Unique tomes run" value={formattedData?.tomeUsage?.length} loading={loading} />
            </div>
            <div className="col-span-1 md:col-span-2 ">
                <GroupBarChart data={formattedData?.groupUsage || []} loading={loading} hosts={hosts} />
            </div>
            <div className="col-span-1 md:col-span-2">
                <TomeBarChart data={formattedData?.tomeUsage || []} loading={loading} />
            </div>
        </div>
    )
}
export default EnvironmentCard;
