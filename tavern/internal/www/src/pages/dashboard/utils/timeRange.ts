import moment from "moment";


export interface TimeRangeConfig {
    label: string;
    granularity_seconds: number;
    daysBack: number;
    formatString: string;
    tickInterval: number;
}

export type TimeRange = "today" | "last3days" | "week" | "month";

export const TIME_RANGES: TimeRange[] = ["today", "last3days", "week", "month"];

export const computeTimeWindow = (config: TimeRangeConfig) => {
    const unit = config.granularity_seconds < 86400 ? "hour" : "day";
    const stop = moment().startOf(unit).add(1, unit);
    const start = moment().subtract(config.daysBack, "days").startOf(unit);
    return { start, stop };
};

export const TIME_RANGE_CONFIG: Record<TimeRange, TimeRangeConfig> = {
    today: {
        label: "1 Day",
        granularity_seconds: 3600,
        daysBack: 1,
        formatString: "h A",
        tickInterval: 1,
    },
    last3days: {
        label: "3 Days",
        granularity_seconds: 3600,
        daysBack: 3,
        formatString: "ddd h A",
        tickInterval: 6,
    },
    week: {
        label: "7 Days",
        granularity_seconds: 86400,
        daysBack: 7,
        formatString: "ddd",
        tickInterval: 1,
    },
    month: {
        label: "30 Days",
        granularity_seconds: 86400,
        daysBack: 30,
        formatString: "MMM D",
        tickInterval: 1,
    },
};

export const beaconComputeTimeWindow = (config: TimeRangeConfig) => {
    if (config.granularity_seconds >= 86400) {
        const stop = moment().startOf("day").add(1, "day");
        const start = moment().subtract(config.daysBack, "days").startOf("day");
        return { start, stop };
    }
    const nowSeconds = moment().unix();
    const stopSeconds = (Math.floor(nowSeconds / config.granularity_seconds) + 1) * config.granularity_seconds;
    const rawStartSeconds = moment().subtract(config.daysBack, "days").unix();
    const startSeconds = Math.floor(rawStartSeconds / config.granularity_seconds) * config.granularity_seconds;
    return { start: moment.unix(startSeconds), stop: moment.unix(stopSeconds) };
};

export const BEACON_TIME_RANGE_CONFIG: Record<TimeRange, TimeRangeConfig> = {
    today: {
        label: "1 Day",
        granularity_seconds: 30,
        daysBack: 1,
        formatString: "h:mm A",
        tickInterval: 120,
    },
    last3days: {
        label: "3 Days",
        granularity_seconds: 3600,
        daysBack: 3,
        formatString: "ddd h A",
        tickInterval: 1,
    },
    week: {
        label: "7 Days",
        granularity_seconds: 14400,
        daysBack: 7,
        formatString: "ddd h A",
        tickInterval: 1,
    },
    month: {
        label: "30 Days",
        granularity_seconds: 86400,
        daysBack: 30,
        formatString: "MMM D",
        tickInterval: 1,
    },
};
