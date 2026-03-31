import { FC } from "react";
import {
    BarChart,
    Bar,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    Legend,
    ResponsiveContainer,
} from "recharts";
import { Tab, TabGroup, TabList } from "@headlessui/react";
import { getTacticColor } from "../../../utils/utils";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import { CustomTooltip } from "./CustomTooltip";
import { CustomLegend } from "./CustomLegend";
import { TIME_RANGES, TIME_RANGE_CONFIG } from "./config";
import { useQuestTimelineChart } from "./useQuestTimelineChart";

export const QuestTimelineChart: FC = () => {
    const {
        selectedIndex,
        chartData,
        activeTactics,
        loading,
        error,
        handleTabChange,
    } = useQuestTimelineChart();

    if (loading && !chartData.length) {
        return (
            <div className="bg-white rounded-lg border border-gray-200 p-6">
                <div className="h-80">
                    <EmptyState
                        type={EmptyStateType.loading}
                        label="Loading quest timeline..."
                    />
                </div>
            </div>
        );
    }

    if (error) {
        return (
            <div className="bg-white rounded-lg border border-gray-200 p-6">
                <div className="h-80">
                    <EmptyState
                        type={EmptyStateType.error}
                        label="Failed to load quest timeline"
                        details={error.message}
                    />
                </div>
            </div>
        );
    }

    return (
        <div className="bg-white rounded-lg border border-gray-200 p-6">
            <div className="flex flex-row justify-between items-center mb-6">
                <h3 className="text-lg font-semibold text-gray-900">Quest Timeline</h3>

                <TabGroup selectedIndex={selectedIndex} onChange={handleTabChange}>
                    <TabList className="flex rounded-lg bg-gray-100 p-1">
                        {TIME_RANGES.map((range) => (
                            <Tab
                                key={range}
                                className={({ selected }) =>
                                    `px-4 py-1.5 text-sm font-medium rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-purple-500 focus:ring-offset-2 ${
                                        selected
                                            ? "bg-white text-purple-700 shadow-sm"
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

            {chartData.length === 0 ? (
                <div className="h-80">
                    <EmptyState
                        type={EmptyStateType.noData}
                        label="No quest data available"
                        details="Create some quests to see the timeline"
                    />
                </div>
            ) : (
                <div className="h-80">
                    <ResponsiveContainer width="100%" height="100%">
                        <BarChart
                            data={chartData}
                            margin={{ top: 20, right: 30, left: 20, bottom: 5 }}
                        >
                            <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
                            <XAxis
                                dataKey="displayLabel"
                                tick={{ fill: "#6b7280", fontSize: 12 }}
                                tickLine={{ stroke: "#e5e7eb" }}
                                axisLine={{ stroke: "#e5e7eb" }}
                            />
                            <YAxis
                                tick={{ fill: "#6b7280", fontSize: 12 }}
                                tickLine={{ stroke: "#e5e7eb" }}
                                axisLine={{ stroke: "#e5e7eb" }}
                                allowDecimals={false}
                            />
                            <Tooltip content={<CustomTooltip />} />
                            <Legend content={<CustomLegend />} />

                            {activeTactics.map((tactic, index) => (
                                <Bar
                                    key={tactic}
                                    dataKey={tactic}
                                    stackId="tactics"
                                    fill={getTacticColor(tactic)}
                                    name={tactic}
                                    radius={
                                        index === activeTactics.length - 1
                                            ? [4, 4, 0, 0]
                                            : [0, 0, 0, 0]
                                    }
                                />
                            ))}
                        </BarChart>
                    </ResponsiveContainer>
                </div>
            )}
        </div>
    );
};

export default QuestTimelineChart;
