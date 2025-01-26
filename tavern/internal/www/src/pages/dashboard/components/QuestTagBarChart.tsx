import React from 'react';
import { useNavigate } from 'react-router-dom';
import { BarChart, Bar, Rectangle, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, Cell } from 'recharts';

import { EmptyState, EmptyStateType } from '../../../components/tavern-base-ui/EmptyState';
import { TaskChartKeys } from '../../../utils/enums';


const TagBarChart = ({ data, loading, tagKind, children }: { data: Array<any>, loading: boolean, tagKind: string, children?: React.ReactNode }) => {
    const navigation = useNavigate();

    if (loading) {
        return <EmptyState type={EmptyStateType.loading} label="Formatting group data..." />
    }

    const height = data.length * 40 < 320 ? 320 : data.length * 40;

    const handleBarClick = (_: any, index?: any) => {
        const item = data[index];
        if (item.name === "undefined") {
            return null;
        }
        navigation("/quests", {
            state: [{
                'label': item?.name,
                'kind': tagKind,
                'name': item?.name,
                'value': item?.id
            }]
        })
    };

    return (
        <div className=" flex flex-col gap-6 w-full h-full">
            <div className='max-h-56 overflow-y-scroll'>
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
                            <YAxis type="category" dataKey="name" width={100} interval={0} />
                            <Tooltip />
                            <Legend />
                            <Bar stackId="a" dataKey={TaskChartKeys.taskNoError} fill="#553C9A" onClick={handleBarClick} activeBar={<Rectangle fill="#805AD5" stroke="#322659" />}>
                                {data.map((_, index) => (
                                    <Cell
                                        cursor="pointer"
                                        fill="#553C9A"
                                        stroke="#322659"
                                        key={`bar-cell-group-task-${index}`}
                                    />
                                ))}
                            </Bar>
                            <Bar stackId="a" dataKey={TaskChartKeys.taskError} fill="#E53E3E" onClick={handleBarClick} activeBar={<Rectangle fill="#F56565" stroke="#822727" />}>
                                {data.map((_, index) => (
                                    <Cell
                                        cursor="pointer"
                                        fill="#E53E3E"
                                        stroke="#E53E3E"
                                        key={`bar-cell-group-task-error-${index}`}
                                    />
                                ))}
                            </Bar>
                        </BarChart>
                    </ResponsiveContainer>
                </div>
            </div>
            {children && children}
        </div>
    );
}
export default TagBarChart;
