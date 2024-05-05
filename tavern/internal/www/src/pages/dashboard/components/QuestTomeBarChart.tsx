import React from 'react';
import { BarChart, Bar, Rectangle, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';

import { EmptyState, EmptyStateType } from '../../../components/tavern-base-ui/EmptyState';
import { TaskChartKeys } from '../../../utils/enums';


const TomeBarChart = ({ data, loading }: { data: Array<any>, loading: boolean }) => {

    if (loading) {
        return <EmptyState type={EmptyStateType.loading} label="Formatting tome data..." />
    }

    const height = data.length * 40 < 320 ? 320 : data.length * 40;

    return (
        <div style={{ height: `${height}px` }}>
            <ResponsiveContainer width="100%" height="100%">
                <BarChart
                    layout='vertical'
                    data={data}
                    margin={{
                        top: 5,
                        left: 5,
                        right: 5,
                        bottom: 5,
                    }}
                >
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis type="number" />
                    <YAxis type="category" dataKey="name" width={300} interval={0} />
                    <Tooltip />
                    <Legend />
                    <Bar stackId="a" dataKey={TaskChartKeys.taskNoError} fill="#553C9A" activeBar={<Rectangle fill="#805AD5" stroke="#322659" />} />
                    <Bar stackId="a" dataKey={TaskChartKeys.taskError} fill="#E53E3E" activeBar={<Rectangle fill="#F56565" stroke="#822727" />} />
                </BarChart>
            </ResponsiveContainer>
        </div>
    );
}
export default TomeBarChart;
