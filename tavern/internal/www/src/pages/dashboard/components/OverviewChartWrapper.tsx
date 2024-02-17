import React from "react";
import { HostType, Task } from "../../../utils/consts";

import { useOverviewData } from "../hook/useOverviewData";

import AccessCard from "./AccessCard";
import EnvironmentCard from "./EnvironmentCard";
import QuestCard from "./QuestCard";

const OverviewChartWrapper = ({ data, hosts }: { data: Array<Task>, hosts: Array<HostType> }) => {
    const { loading, formattedData } = useOverviewData(data);

    return (
        <div className="my-4 flex flex-col gap-4">
            <QuestCard formattedData={formattedData} loading={loading} />
            <AccessCard hosts={hosts} />
            {data.length > 0 && <EnvironmentCard formattedData={formattedData} loading={loading} hosts={hosts} />}
        </div>
    );
}
export default OverviewChartWrapper;
