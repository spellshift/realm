import { FC, ReactNode } from "react";
import { TrendingUp, TrendingDown } from "lucide-react";

type TrendDirection = "up" | "down";

interface MetricCardProps {
    label?: string;
    count: number | string;
    timeframe?: string;
    trend?: TrendDirection;
    trendValue?: number | string;
}

const TREND_ICON: Record<TrendDirection, ReactNode> = {
    up: <TrendingUp className="w-4 h-4 text-green-600" />,
    down: <TrendingDown className="w-4 h-4 text-red-600" />,
};

export const MetricCard: FC<MetricCardProps> = ({ label, count, timeframe, trend, trendValue }) => {
    return (
        <div className="flex flex-col gap-1">
            <div className='text-xs uppercase text-gray-600 h-3'>
                {label}
            </div>
            <div className="flex flex-row gap-2 items-end">
                <div className="text-3xl font-semibold text-gray-900">{count}</div>
                {trend && (
                    <div className="flex flex-col">
                        <div className="flex flex-row items-center gap-2">
                            {TREND_ICON[trend]}
                            <div className="text-sm">
                                {trendValue}
                            </div>
                        </div>
                        <div className="-mt-1 text-xs text-gray-600">{timeframe}</div>
                    </div>
                )}
            </div>
        </div>
    );
};

export default MetricCard;
