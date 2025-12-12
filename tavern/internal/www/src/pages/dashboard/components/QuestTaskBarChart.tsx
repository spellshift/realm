import React from 'react';
import { XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, BarChart, Bar } from 'recharts';

import EmptyStateNoQuests from '../../../components/empty-states/EmptyStateNoQuests';
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import { getTacticColor } from '../../../utils/utils';


const TaskBarChart = ({ data, taskTactics, loading }: { data: Array<any>, taskTactics: Array<string>, loading: boolean }) => {

    if (loading) {
        return <EmptyState type={EmptyStateType.loading} label="Formatting task data..." />
    }
    if (!data || data?.length <= 0) {
        return (
            <EmptyStateNoQuests />
        )
    }

    return (
        <ResponsiveContainer width="100%" height="100%">
            <BarChart
                width={500}
                height={300}
                data={data}
                margin={{
                    top: 5,
                    right: 5,
                    left: 5,
                    bottom: 5,
                }}
            >
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="label" />
                <YAxis />
                <Tooltip />
                <Legend />
                {taskTactics.map((tactic: any, index: number) => {
                    return (
                        <Bar key={`${tactic}_${index}`} type="monotone" dataKey={tactic} stackId="a" fill={getTacticColor(tactic.toUpperCase())} />
                    )
                })}
            </BarChart>
        </ResponsiveContainer>
    );
}
export default TaskBarChart;
