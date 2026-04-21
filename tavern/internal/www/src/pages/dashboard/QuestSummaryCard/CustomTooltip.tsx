import { FC } from "react";
import { TomeTactic } from "../../../utils/enums";

interface CustomTooltipProps {
    active?: boolean;
    payload?: Array<{ name: string; value: number; fill: string }>;
    label?: string;
}

export const CustomTooltip: FC<CustomTooltipProps> = ({ active, payload, label }) => {
    if (!active || !payload?.length) return null;

    const sortedPayload = [...payload]
        .filter((entry) => entry.value > 0)
        .sort((a, b) => b.value - a.value);

    if (!sortedPayload.length) return null;

    return (
        <div className="bg-white border border-gray-200 rounded-lg shadow-lg p-3">
            <p className="font-semibold text-gray-900 mb-2">{label}</p>
            <div className="space-y-1">
                {sortedPayload.map((entry) => (
                    <div key={entry.name} className="flex items-center gap-2 text-sm">
                        <div
                            className="w-3 h-3 rounded-sm"
                            style={{ backgroundColor: entry.fill }}
                        />
                        <span className="text-gray-800">
                            {TomeTactic[entry.name as keyof typeof TomeTactic] || entry.name}:
                        </span>
                        <span className="font-medium text-gray-900">{entry.value}</span>
                    </div>
                ))}
            </div>
        </div>
    );
};
