export interface TimelineBucket {
    count: number;
    startTimestamp: string;
}

export interface TimelineDataPoint {
    timestamp: string;
    displayLabel: string;
    total: number;
}

export interface ChartDataPoint extends TimelineDataPoint {
    [key: string]: string | number;
}

export interface QuestTimelineTacticBucket {
    tactic: string;
    count: number;
}

export interface QuestTimelineBucket extends TimelineBucket {
    groupByTactic: QuestTimelineTacticBucket[];
}

export interface QuestTimelineChartResponse {
    metrics: {
        questTimelineChart: QuestTimelineBucket[];
    };
}

export interface BeaconTimelineChartResponse {
    metrics: {
        beaconTimelineChart: TimelineBucket[];
    };
}
