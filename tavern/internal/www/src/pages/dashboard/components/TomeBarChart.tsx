import { BookOpenIcon } from '@heroicons/react/24/outline';
import React, { useState } from 'react';
import { BarChart, Bar, Rectangle, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import { EmptyState, EmptyStateType } from '../../../components/tavern-base-ui/EmptyState';


const TomeBarChart = ({ data, loading }: { data: Array<any>, loading: boolean }) => {

    if (loading) {
        return <EmptyState type={EmptyStateType.loading} label="Formatting tome data..." />
    }

    const height = data.length * 40 < 320 ? 320 : data.length * 40;

    return (
        <div className=" bg-white rounded-lg shadow-lg flex flex-col gap-6 w-full h-full p-4">
            <div className='flex flex-row gap-4 items-center'>
                <div className="rounded-md bg-purple-900 p-4">
                    <BookOpenIcon className="text-white w-8 h-8" />
                </div>
                <div className='flex flex-col'>
                    <h2 className="text-lg font-semibold text-gray-900">Unique tomes run</h2>
                    <h3 className='text-lg'>{data.length.toLocaleString()}</h3>
                </div>
            </div>
            <div className='max-h-80 overflow-y-scroll'>
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
