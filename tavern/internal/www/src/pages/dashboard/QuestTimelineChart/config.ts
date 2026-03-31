import { gql } from "@apollo/client";
import { TomeTactic } from "../../../utils/enums";
import { TimeRange, TimeRangeConfig } from "./types";

export const TIME_RANGES: TimeRange[] = ["today", "last3days", "week", "month"];

export const TIME_RANGE_CONFIG: Record<TimeRange, TimeRangeConfig> = {
    today: {
        label: "1 Day",
        granularity_seconds: 3600, 
        daysBack: 1,
        formatString: "h A",
    },
    last3days: {
        label: "3 Days",
        granularity_seconds: 7200,
        daysBack: 3,
        formatString: "ddd h A",
    },
    week: {
        label: "7 Days",
        granularity_seconds: 86400,
        daysBack: 7,
        formatString: "ddd",
    },
    month: {
        label: "30 Days",
        granularity_seconds: 86400,
        daysBack: 30,
        formatString: "MMM D",
    },
};

export const ALL_TACTICS = Object.keys(TomeTactic) as Array<keyof typeof TomeTactic>;

export const GET_QUEST_TIMELINE_CHART = gql`
    query GetQuestTimelineChart($start: Time!, $end: Time, $granularity_seconds: Int!) {
        metrics {
            questTimelineChart(start: $start, end: $end, granularity_seconds: $granularity_seconds) {
                count
                startTimestamp
                groupByTactic {
                    tactic
                    count
                }
            }
        }
    }
`;