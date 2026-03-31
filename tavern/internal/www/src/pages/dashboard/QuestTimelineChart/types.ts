export type TimeRange = "today" | "last3days" | "week" | "month";

export interface TimeRangeConfig {
    label: string;
    granularity_seconds: number;
    daysBack: number;
    formatString: string;
}

export interface QuestTimelineTacticBucket {
    tactic: string;
    count: number;
}

export interface QuestTimelineBucket {
    count: number;
    startTimestamp: string;
    groupByTactic: QuestTimelineTacticBucket[];
}

export interface QuestTimelineChartResponse {
    metrics: {
        questTimelineChart: QuestTimelineBucket[];
    };
}

export interface ChartDataPoint {
    displayLabel: string;
    timestamp: string;
    total: number;
    [key: string]: string | number;
}
