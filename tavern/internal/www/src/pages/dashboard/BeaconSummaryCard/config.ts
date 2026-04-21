import { gql } from "@apollo/client";

export const GET_BEACON_TIMELINE_CHART = gql`
    query GetBeaconTimelineChart($start: Time!, $end: Time, $granularity_seconds: Int!) {
        metrics {
            beaconTimelineChart(start: $start, end: $end, granularity_seconds: $granularity_seconds) {
                count
                startTimestamp
            }
        }
    }
`;
