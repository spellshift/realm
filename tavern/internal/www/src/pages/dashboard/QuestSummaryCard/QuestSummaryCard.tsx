import { FC } from "react";
import { Tab, TabGroup, TabList } from "@headlessui/react";
import { useQuestTimelineChart } from "./useQuestTimelineChart";
import { QuestTimelineChart } from "./QuestTimelineChart";
import { MetricCard } from "../MetricCard";
import { TIME_RANGE_CONFIG, TIME_RANGES } from "../utils/timeRange";

export const QuestSummaryCard: FC = () => {
    const {
        selectedIndex,
        chartData,
        activeTactics,
        questMetric,
        loading,
        error,
        handleTabChange,
    } = useQuestTimelineChart();

    return (
        <div className="bg-white rounded-lg border border-gray-200 py-2 px-6 flex flex-col gap-4">
            <div className="flex flex-row justify-between items-center">
                <h3 className="text-lg font-semibold text-gray-900">Quest Timeline</h3>

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

            <MetricCard label="Quests" count={questMetric.count} trend={questMetric.trend} timeframe={questMetric.timeframe} trendValue={questMetric.trendValue ?? undefined} />

            <div className="h-64">
                <QuestTimelineChart
                    loading={loading}
                    chartData={chartData}
                    error={error}
                    activeTactics={activeTactics}
                />
            </div>
        </div>
    );
};

export default QuestSummaryCard;
