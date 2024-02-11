import { ClipboardDocumentListIcon } from '@heroicons/react/24/outline';
import { XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, BarChart, Bar } from 'recharts';

import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import { getTacticColor } from '../../../utils/utils';


const TaskBarChart = ({ total, data, taskTactics, loading }: { total: number, data: Array<any>, taskTactics: Array<string>, loading: boolean }) => {

    if (loading) {
        return <EmptyState type={EmptyStateType.loading} label="Formatting tome data..." />
    }

    return (
        <div className=" bg-white rounded-lg shadow-lg flex flex-col gap-6 w-full h-full p-4">
            <div className='flex flex-row gap-4 items-center'>
                <div className="rounded-md bg-purple-900 p-4">
                    <ClipboardDocumentListIcon className="text-white w-8 h-8" />
                </div>
                <div className='flex flex-col'>
                    <h2 className="text-lg font-semibold text-gray-900">Tasks created</h2>
                    <h3 className='text-lg'>{total.toLocaleString()}</h3>
                </div>
            </div>
            <div className='h-80 overflow-y-scroll'>
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
                                <Bar type="monotone" dataKey={tactic} stackId="a" fill={getTacticColor(tactic.toUpperCase())} />
                            )
                        })}
                    </BarChart>
                </ResponsiveContainer>
            </div>
        </div>
    );
}
export default TaskBarChart;
