import { FC } from "react";
import { TomeTactic } from "../../../utils/enums";

interface CustomLegendProps {
    payload?: Array<{ value: string; color: string }>;
}

export const CustomLegend: FC<CustomLegendProps> = ({ payload }) => {
    if (!payload?.length) return null;

    return (
        <div className="flex flex-wrap justify-center gap-x-4 gap-y-2 mt-4">
            {payload.map((entry) => (
                <div key={entry.value} className="flex items-center gap-1.5 text-xs">
                    <div
                        className="w-2.5 h-2.5 rounded-sm"
                        style={{ backgroundColor: entry.color }}
                    />
                    <span className="text-gray-600">
                        {TomeTactic[entry.value as keyof typeof TomeTactic] || entry.value}
                    </span>
                </div>
            ))}
        </div>
    );
};
