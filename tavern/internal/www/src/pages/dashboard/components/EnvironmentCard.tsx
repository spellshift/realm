import React from "react";
import DashboardStatistic from "./DashboardStatistic";

import TomeBarChart from "./TomeBarChart";
import TagBarChart from "./TagBarChart";
import TargetReccomendation from "./TargetRecommendation";

const EnvironmentCard = ({ formattedData, hosts, loading }: { formattedData: any, hosts: Array<any>, loading: boolean }) => {

    return (
        <div className="grid grid-cols-1 md:grid-cols-5 gap-6 bg-white rounded-lg shadow-lg p-4">
            <h3 className="text-xl col-span-1 md:col-span-5">
                Engagement breakdown
            </h3>
            <div className="col-span-1 flex flex-row md:flex-col gap-4 flex-wrap">
                <DashboardStatistic label="Total groups" value={formattedData?.groupUsage?.length} loading={loading} />
                <DashboardStatistic label="Total services" value={formattedData?.serviceUsage.length} loading={loading} />
                <DashboardStatistic label="Unique tomes run" value={formattedData?.tomeUsage?.length} loading={loading} />
            </div>
            <div className="col-span-4 grid grid-cols-4 gap-4">
                <div className="col-span-2">
                    <TagBarChart data={formattedData?.groupUsage || []} loading={loading} tagKind="group">
                        <TargetReccomendation data={formattedData?.groupUsage || []} tagKind="group" hosts={hosts} />
                    </TagBarChart>
                </div>
                <div className="col-span-2">
                    <TagBarChart data={formattedData?.serviceUsage || []} loading={loading} tagKind="service">
                        <TargetReccomendation data={formattedData?.serviceUsage || []} tagKind="service" hosts={hosts} />
                    </TagBarChart>
                </div>
                <div className="col-span-4">
                    <TomeBarChart data={formattedData?.tomeUsage || []} loading={loading} />
                </div>
                <div className="col-span-2">

                </div>
            </div>
        </div>
    )
}
export default EnvironmentCard;
