import { useState } from "react";
import EmptyStateNoBeacon from "../../../components/empty-states/EmptyStateNoBeacon";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import { useHostAcitvityData } from "../hook/useHostActivityData";
import DashboardStatistic from "./DashboardStatistic";
import AccessHostActivityTable from "./AccessHostActivityTable";
import SingleDropdownSelector from "../../../components/tavern-base-ui/SingleDropdownSelector";

const AccessCard = ({ hosts }: { hosts: any }) => {
    const { loading, hostActivity, onlineHostCount, offlineHostCount } = useHostAcitvityData(hosts);

    const [selectedOption, setSelectedOption] = useState({
        "label": "Group",
        "value": "group"
    });

    const accessOptions = [
        {
            label: "Group",
            value: "group",
        },
        {
            label: "Service",
            value: "service",
        },
        {
            label: "Platform",
            value: "platform"
        }
    ];

    if (!hosts && hosts?.length < 1) {
        <EmptyStateNoBeacon />
    }

    return (
        <div className="grid grid-cols-1 md:grid-cols-5  gap-6  bg-white rounded-lg shadow-lg p-4">
            <h3 className="text-xl col-span-1 md:col-span-5">
                Access status
            </h3>
            <div className="col-span-1 flex flex-row md:flex-col gap-4 flex-wrap">
                <DashboardStatistic label="Online hosts" value={onlineHostCount} loading={loading} />
                <DashboardStatistic label="Offline hosts" value={offlineHostCount} loading={loading} />
            </div>
            <div className="col-span-1 md:col-span-4">
                {loading ? (
                    <EmptyState type={EmptyStateType.loading} label="Formatting host data..." />
                ) : (!hostActivity) ? (
                    <EmptyState type={EmptyStateType.noData} label="Unable to format access by group tag" />
                ) : (
                    <div className="flex flex-col w-full h-full gap-4">
                        <div className='flex flex-row gap-2 items-center'>
                            <h2 className="text-lg">Access by</h2>
                            <div>
                                <SingleDropdownSelector setSelectedOption={setSelectedOption} options={accessOptions} label="accessDropdown" />
                            </div>
                        </div>
                        <div className='h-80 overflow-y-scroll'>
                            <AccessHostActivityTable hostActivity={hostActivity[selectedOption.value]} term={selectedOption.value} />
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}
export default AccessCard;
