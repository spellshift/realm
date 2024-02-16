import { Button } from '@chakra-ui/react';
import { BookOpenIcon, UserCircleIcon } from '@heroicons/react/24/outline';
import React, { useCallback, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { BarChart, Bar, Rectangle, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, Cell } from 'recharts';
import { EmptyState, EmptyStateType } from '../../../components/tavern-base-ui/EmptyState';
import { TomeTag } from '../../../utils/consts';
import { getOfflineOnlineStatus } from '../../../utils/utils';


const GroupBarChart = ({ data, loading, hosts }: { data: Array<any>, loading: boolean, hosts: Array<any> }) => {
    const navigation = useNavigate();

    if (loading) {
        return <EmptyState type={EmptyStateType.loading} label="Formatting group data..." />
    }

    const height = data.length * 40 < 320 ? 320 : data.length * 40;
    const groupWithFewestTasks = data.length > 0 ? data[0] : {};

    const getTotalActiveBeaconsForGroup = () => {
        const returnedValue = hosts.reduce((acc, curr) => {
            const matchesGroup = curr?.tags.find((tag: TomeTag) => { return tag.name === groupWithFewestTasks.name });
            const beaconStatus = getOfflineOnlineStatus(curr.beacons || [])
            if (matchesGroup) {
                return acc += beaconStatus.online;
            }
            return acc;
        }, 0);
        return returnedValue;
    };

    const activeBeaconForGroupWithFewestTasks = getTotalActiveBeaconsForGroup();

    const average = data.reduce((a: any, b: any) => { return a + b["task count"] }, 0) / data.length;


    const handleClickQuestDetails = (item: any) => {
        navigation("/tasks", {
            state: [{
                'label': item?.name,
                'kind': 'group',
                'name': item?.name,
                'value': item?.id
            }]
        })
    }

    const handleBarClick = (_: any, index?: any) => {
        const item = data[index];
        navigation("/tasks", {
            state: [{
                'label': item?.name,
                'kind': 'group',
                'name': item?.name,
                'value': item?.id
            }]
        })
    };

    return (
        <div className=" bg-white rounded-lg shadow-lg flex flex-col gap-6 w-full h-full p-4">
            <div className='flex flex-row gap-4 items-center'>
                <div className="rounded-md bg-purple-900 p-4">
                    <UserCircleIcon className="text-white w-8 h-8" />
                </div>
                <div className='flex flex-col'>
                    <h2 className="text-lg font-semibold text-gray-900">Avg task per group</h2>
                    <h3 className='text-lg'>{average.toFixed(0)} </h3>
                </div>
            </div>
            <div className='max-h-56 overflow-y-scroll'>
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
                            <YAxis type="category" dataKey="name" width={100} interval={0} />
                            <Tooltip />
                            <Legend />
                            <Bar dataKey="task count" fill="#553C9A" onClick={handleBarClick}>
                                {data.map((_, index) => (
                                    <Cell
                                        cursor="pointer"
                                        fill="#805AD5"
                                        stroke="#322659"
                                        key={`bar-cell-group-task-${index}`}
                                    />
                                ))}
                            </Bar>
                        </BarChart>
                    </ResponsiveContainer>
                </div>
            </div>
            <div className='flex flex-col border-l-4 border-purple-900 px-4 py-2 rounded'>
                <h4 className="font-semibold text-gray-900">Consider targeting the group with fewest tasks</h4>
                <p className='text-sm'>{groupWithFewestTasks.name} has {groupWithFewestTasks["task count"]} task run and {activeBeaconForGroupWithFewestTasks} online beacons</p>
                <div className='flex flex-row gap-4 mt-2'>
                    <Button size="sm" variant="link" colorScheme="purple" onClick={() => {
                        handleClickQuestDetails(groupWithFewestTasks)
                    }}>
                        See quest details
                    </Button>
                </div>
            </div>
        </div>
    );
}
export default GroupBarChart;
