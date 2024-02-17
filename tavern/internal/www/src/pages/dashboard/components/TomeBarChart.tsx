import React from 'react';
import { BarChart, Bar, Rectangle, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';

import { EmptyState, EmptyStateType } from '../../../components/tavern-base-ui/EmptyState';


const TomeBarChart = ({ data, loading }: { data: Array<any>, loading: boolean }) => {

    if (loading) {
        return <EmptyState type={EmptyStateType.loading} label="Formatting tome data..." />
    }

    const height = data.length * 40 < 320 ? 320 : data.length * 40;

    return (
        <div className="flex flex-col gap-6 w-full h-full">
            <div className='flex flex-row gap-4 items-center'>
                <h2 className="text-lg">Tasks by tome name</h2>
            </div>
            <div className='max-h-56 overflow-y-scroll '>
                <div style={{ height: `${height}px` }}>
                    <ResponsiveContainer width="100%" height="100%">
                        <BarChart
                            layout='vertical'
                            width={500}
                            height={300}
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
                            <Bar dataKey="task count" fill="#553C9A" activeBar={<Rectangle fill="#805AD5" stroke="#322659" />} />
                        </BarChart>
                    </ResponsiveContainer>
                </div>
            </div>
        </div>
    );
}
export default TomeBarChart;
