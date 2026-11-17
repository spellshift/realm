import { Pencil, Trash2 } from "lucide-react";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import { TRANSPORT_TYPES, TransportConfig, TransportTypeValue } from "../types";
import { transportExtraSummary } from "./transportForm";

export const transportLabel = (type: TransportTypeValue) => TRANSPORT_TYPES.find((option) => option.value === type)?.label ?? "Transport";

type Props = {
    transports: TransportConfig[];
    editingId?: string;
    onEdit?: (transport: TransportConfig) => void;
    onRemove?: (transport: TransportConfig) => void;
    showDetails?: boolean;
    variant?: "editor" | "summary";
};

export const TransportList = ({ transports, editingId, onEdit, onRemove, showDetails = false, variant = "editor" }: Props) => {
    const showActions = !!onEdit && !!onRemove;
    const isSummary = variant === "summary";
    const cellClass = isSummary ? "px-4 py-4" : "px-3 py-3";

    if (transports.length === 0) {
        return (
            <div className="flex items-center justify-center rounded-lg border border-dashed border-gray-300 bg-gray-50 py-8 text-sm text-gray-500">
                No transports configured.
            </div>
        );
    }

    return (
        <div className="min-w-0 overflow-x-auto rounded-lg border border-gray-200 bg-white">
            <table className={`w-full table-fixed text-sm ${isSummary ? "min-w-[300px]" : "min-w-[560px]"}`}>
                <colgroup><col className="w-[34%]" /><col /><col className="w-36" />{showActions && <col className="w-28" />}</colgroup>
                <thead className="border-b border-gray-200 bg-gray-50 text-left text-xs font-medium uppercase tracking-wide text-gray-500">
                    <tr>
                        <th scope="col" className={cellClass}>Type</th>
                        <th scope="col" className={cellClass}>URI</th>
                        <th scope="col" className={cellClass}>Interval</th>
                        {showActions && <th scope="col" className={cellClass}><span className="sr-only">Actions</span></th>}
                    </tr>
                </thead>
                <tbody className="divide-y divide-gray-200">
                    {transports.map((transport) => {
                        const selected = editingId === transport.id;
                        const label = transportLabel(transport.type);
                        return <tr key={transport.id} className={`bg-white ${isSummary ? "h-20" : ""}`}>
                            <td className={`${cellClass} ${selected ? "border-l-[3px] border-purple-600 pl-[9px]" : ""}`}><Badge badgeStyle={{ color: "gray" }}>{label}</Badge></td>
                            <td className={`break-all text-gray-900 ${cellClass}`}>
                                <div>{transport.uri}</div>
                                {showDetails && transportExtraSummary(transport).map((detail) => (
                                    <div key={detail} className="text-xs text-gray-500">{detail}</div>
                                ))}
                            </td>
                            <td className={`text-gray-900 ${cellClass}`}>{transport.interval}s</td>
                            {showActions && <td className={cellClass}><div className="flex items-center justify-end gap-2">
                                <button type="button" className="rounded p-1 text-gray-400 hover:text-gray-600" aria-label={`Edit ${label} transport`}
                                    onClick={() => onEdit(transport)}><Pencil className="h-4 w-4" /></button>
                                <button type="button" className="rounded p-1 text-red-700 hover:text-red-800" aria-label={`Remove ${label} transport`}
                                    onClick={() => onRemove(transport)}><Trash2 className="h-4 w-4" /></button>
                            </div></td>}
                        </tr>;
                    })}
                </tbody>
            </table>
        </div>
    );
};
