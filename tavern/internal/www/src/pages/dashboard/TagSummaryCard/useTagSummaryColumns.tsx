import { useCallback, useMemo } from "react";
import { ColumnDef } from "@tanstack/react-table";
import { formatDistance } from "date-fns";
import { useNavigate } from "react-router-dom";
import Button from "../../../components/tavern-base-ui/button/Button";
import { useFilters } from "../../../context/FilterContext";
import { OnlineOfflineFilterType } from "../../../utils/enums";
import { FilterBarOption } from "../../../utils/interfacesUI";
import { ONLINE_OFFLINE_OPTIONS } from "../../../components/beacon-filter-bar/constants";
import { TagRow } from "./types";

export function useTagSummaryColumns(): ColumnDef<TagRow>[] {
    const navigate = useNavigate();
    const { updateFilters, setIsLocked } = useFilters();

    const tagFilter = useCallback((row: TagRow): FilterBarOption => ({
        kind: row.tagKind,
        id: row.tagId,
        name: row.tagName,
        value: row.tagId,
        label: row.tagName,
    }), []);

    const handleQuestCellClick = useCallback((row: TagRow) => {
        setIsLocked(true);
        updateFilters({ beaconFields: [tagFilter(row)] });
        navigate('/quests');
    }, [navigate, updateFilters, setIsLocked, tagFilter]);

    const handleHostStatusClick = useCallback((row: TagRow, statusId: OnlineOfflineFilterType) => {
        const statusFilter = ONLINE_OFFLINE_OPTIONS.find((opt) => opt.id === statusId);
        if (!statusFilter) return;
        setIsLocked(true);
        updateFilters({ beaconFields: [tagFilter(row), statusFilter] });
        navigate('/hosts');
    }, [navigate, updateFilters, setIsLocked, tagFilter]);

    return useMemo(() => [
        {
            id: "tagName",
            header: "Tag",
            accessorFn: (row) => row.tagName,
            enableSorting: true,
        },
        {
            id: "onlineHosts",
            header: "Online Hosts",
            accessorFn: (row) => row.onlineHosts,
            enableSorting: true,
        },
        {
            id: "lostHosts",
            header: "Hosts Lost",
            accessorFn: (row) => row.lostHosts,
            enableSorting: true,
            cell: (info) => {
                const count = info.getValue() as number;
                const row = info.row.original;
                if (count === 0) return <span>{count}</span>;
                return (
                    <Button
                        buttonVariant="solid"
                        buttonStyle={{ color: "red", size: "sm" }}
                        onClick={(e) => { e.stopPropagation(); handleHostStatusClick(row, OnlineOfflineFilterType.OfflineHost); }}
                    >
                        {count}
                    </Button>
                );
            },
        },
        {
            id: "onlineBeacons",
            header: "Online Beacons",
            accessorFn: (row) => row.onlineBeacons,
            enableSorting: true,
            cell: (info) => {
                const count = info.getValue() as number;
                const row = info.row.original;
                if (count === 0) return <span>{count}</span>;
                return (
                    <Button
                        buttonVariant="solid"
                        buttonStyle={{ color: "gray", size: "sm" }}
                        onClick={(e) => { e.stopPropagation(); handleHostStatusClick(row, OnlineOfflineFilterType.OnlineBeacons); }}
                    >
                        {count}
                    </Button>
                );
            },
        },
        {
            id: "recentlyLostBeacons",
            header: "Recently Lost Beacons",
            accessorFn: (row) => row.recentlyLostBeacons,
            enableSorting: true,
            cell: (info) => {
                const count = info.getValue() as number;
                const row = info.row.original;
                if (count === 0) return <span>{count}</span>;
                return (
                    <Button
                        buttonVariant="solid"
                        buttonStyle={{ color: "red", size: "sm" }}
                        onClick={(e) => { e.stopPropagation(); handleHostStatusClick(row, OnlineOfflineFilterType.RecentlyLostBeacons); }}
                    >
                        {count}
                    </Button>
                );
            },
        },
        {
            id: "questCount",
            header: "Quests",
            accessorFn: (row) => row.questCount,
            enableSorting: true,
            cell: (info) => {
                const count = info.getValue() as number;
                const row = info.row.original;
                if (count === 0) return <span>{count}</span>;
                return (
                    <Button
                        buttonVariant="solid"
                        buttonStyle={{ color: "gray", size: "sm" }}
                        onClick={(e) => { e.stopPropagation(); handleQuestCellClick(row); }}
                    >
                        {count}
                    </Button>
                );
            },
        },
        {
            id: "lastCallbackAt",
            header: "Last Callback",
            accessorFn: (row) => row.lastCallbackAt,
            enableSorting: true,
            sortingFn: (rowA, rowB) => {
                const a = rowA.original.lastCallbackAt;
                const b = rowB.original.lastCallbackAt;
                if (!a && !b) return 0;
                if (!a) return 1;
                if (!b) return -1;
                return new Date(a) < new Date(b) ? 1 : new Date(a) > new Date(b) ? -1 : 0;
            },
            cell: (info) => {
                const val = info.getValue() as string | null;
                if (!val) return <span className="text-gray-400">Never</span>;
                return <span>{formatDistance(new Date(val), new Date(), { addSuffix: true })}</span>;
            },
        },
    ], [handleQuestCellClick, handleHostStatusClick]);
}
