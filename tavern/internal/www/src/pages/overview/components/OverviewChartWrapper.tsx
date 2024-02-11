import { BookOpenIcon, ClipboardDocumentListIcon } from "@heroicons/react/24/outline";
import { useOverviewData } from "../hook/useOverviewData";
import GroupBarChart from "./GroupBarChart";
import GroupHostActivityTable from "./GroupHostActivityTable";
import TaskBarChart from "./TaskBarChart";
import TomeBarChart from "./TomeBarChart";

const OverviewChartWrapper = ({ data, hosts }: { data: Array<any>, hosts: Array<any> }) => {
    const { loading, formattedData } = useOverviewData(data);

    return (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 my-4">
            <div className="col-span-1">
                <TaskBarChart total={data?.length} data={formattedData?.taskTimelime || []} taskTactics={formattedData.taskTactics} loading={loading} />
            </div>
            <div className="col-span-1">
                <GroupBarChart data={formattedData?.groupUsage || []} loading={loading} hosts={hosts} />
            </div>
            <div className="col-span-1">
                <TomeBarChart data={formattedData?.tomeUsage || []} loading={loading} />
            </div>
            <div className="col-span-1">
                <GroupHostActivityTable hosts={hosts} />
            </div>

        </div>
    );
}
export default OverviewChartWrapper;
