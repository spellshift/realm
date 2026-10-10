import { FC } from "react";
import { Tab, TabGroup, TabList } from "@headlessui/react";
import { TIME_RANGES, TIME_RANGE_CONFIG } from "../utils/timeRange";
import { useBeaconTimelineChart } from "./useBeaconTimelineChart";
import { BeaconTimelineChart } from "./BeaconTimelineChart";
import { MetricCard } from "../MetricCard";

export const BeaconSummaryCard: FC = () => {
    const {
        selectedIndex,
        chartData,
        beaconMetric,
        tickInterval,
        loading,
        error,
        handleTabChange,
    } = useBeaconTimelineChart();

    return (
        <div className="bg-white rounded-lg border border-gray-200 py-2 px-6 flex flex-col gap-4">
            <div className="flex flex-row justify-between items-center">
                <h3 className="text-lg font-semibold text-gray-900">Beacon Timeline</h3>

                <TabGroup selectedIndex={selectedIndex} onChange={handleTabChange}>
                    <TabList className="flex rounded-lg bg-gray-100 p-1">
                        {TIME_RANGES.map((range) => (
                            <Tab
                                key={range}
                                className={({ selected }) =>
                                    `px-4 py-1.5 text-sm font-medium rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-purple-600 focus:ring-offset-2 ${selected
                                        ? "bg-white text-purple-800 semi-bold shadow-sm"
                                        : "text-gray-600 hover:text-gray-900 hover:bg-gray-50"
                                    }`
                                }
                            >
                                {TIME_RANGE_CONFIG[range].label}
                            </Tab>
                        ))}
                    </TabList>
                </TabGroup>
            </div>

            <MetricCard label="Beacons" count={beaconMetric.count} trend={beaconMetric.trend} timeframe={beaconMetric.timeframe} trendValue={beaconMetric.trendValue ?? undefined} />

            <div className="h-64">
                <BeaconTimelineChart
                    loading={loading}
                    chartData={chartData}
                    error={error}
                    tickInterval={tickInterval}
                />
            </div>
        </div>
    );
};

export default BeaconSummaryCard;
