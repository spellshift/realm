import React from 'react';
import { XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, BarChart, Bar } from 'recharts';

import EmptyStateNoQuests from '../../../components/empty-states/EmptyStateNoQuests';
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import { getTacticColor } from '../../../utils/utils';


const TaskBarChart = ({ data, taskTactics, loading }: { data: Array<any>, taskTactics: Array<string>, loading: boolean }) => {

    if (loading) {
        return <EmptyState type={EmptyStateType.loading} label="Formatting tome data..." />
    }
    if (!data || data?.length <= 0) {
        return (
            <EmptyStateNoQuests />
        )
    }

    return (
        <div className="flex flex-col gap-6 w-full h-full">
            <div className='flex flex-row gap-4 items-center'>
                <h2 className="text-lg">Tasks by creation time</h2>
            </div>
            <div className='h-56 overflow-y-scroll'>
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
            </div>
        </div>
    );
}
export default TaskBarChart;
