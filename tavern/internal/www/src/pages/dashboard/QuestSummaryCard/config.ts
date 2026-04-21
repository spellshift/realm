import { gql } from "@apollo/client";
import { TomeTactic } from "../../../utils/enums";

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

export const GET_QUEST_FILTERED_TIMELINE_CHART = gql`
    query GetQuestFilteredTimelineChart($start: Time!, $end: Time, $granularity_seconds: Int!, $where: QuestWhereInput) {
        metrics {
            questTimelineChart(start: $start, end: $end, granularity_seconds: $granularity_seconds, where: $where) {
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
