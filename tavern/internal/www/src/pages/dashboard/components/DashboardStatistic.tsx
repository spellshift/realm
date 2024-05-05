import React from "react";

const DashboardStatistic = (
    {
        label,
        value,
        loading
    }:
        {
            label: string;
            value: number | null;
            loading: boolean
        }
) => {
    return (
        <div className='flex flex-col gap-2'>
            <p className="truncate text-sm font-medium text-gray-500 text-wrap">{label}</p>
            <p className="text-2xl font-semibold text-gray-900">
                {loading ? "-" : value}
            </p>
        </div>
    );
}
export default DashboardStatistic
