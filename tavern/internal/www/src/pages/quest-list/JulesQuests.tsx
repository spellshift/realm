import React, { useCallback, useRef, useState, useEffect, useMemo } from "react";
import { List, AutoSizer } from "react-virtualized";
import { useNavigate } from "react-router-dom";

import { PageWrapper } from "../../components/page-wrapper";
import { PageNavItem } from "../../utils/enums";
import QuestHeader from "./components/QuestHeader";
import { FilterControls, FilterPageType } from "../../context/FilterContext/index";
import { SortingControls } from "../../context/SortContext/index";
import { useJulesQuests } from "./useJulesQuests";
import { QuestRow } from "./components/QuestRow";

// --- Styles ---
const HEADER_CELL_CLASS = "px-4 sm:px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider overflow-hidden text-ellipsis whitespace-nowrap";

// We will use flexbox for columns.
const COL_WIDTHS = {
    chevron: "w-[40px] flex-none flex justify-center",
    name: "flex-[2_1_200px]",
    updated: "w-[120px] flex-none",
    finished: "w-[80px] flex-none",
    output: "w-[80px] flex-none",
    error: "w-[80px] flex-none",
    creator: "w-[150px] flex-none",
};

const JulesQuests = () => {
    const {
        data,
        loading,
        error,
        loadMore,
        hasNextPage
    } = useJulesQuests();

    const navigate = useNavigate();
    const quests = useMemo(() => data?.quests?.edges || [], [data]);
    const rowCount = quests.length;
    const listRef = useRef<any>(null);

    // State for expanded rows
    const [expandedRows, setExpandedRows] = useState<Set<string>>(new Set());
    // Periodically update current date for "5 mins ago"
    const [currentDate, setCurrentDate] = useState(new Date());

    useEffect(() => {
        const timer = setInterval(() => setCurrentDate(new Date()), 60000);
        return () => clearInterval(timer);
    }, []);

    const toggleRow = useCallback((id: string) => {
        setExpandedRows(prev => {
            const next = new Set(prev);
            if (next.has(id)) {
                next.delete(id);
            } else {
                next.add(id);
            }
            return next;
        });
    }, []);

    // Force recompute row heights when expandedRows changes
    useEffect(() => {
        if (listRef.current) {
            listRef.current.recomputeRowHeights();
            listRef.current.forceUpdateGrid(); // Ensure render
        }
    }, [expandedRows]);

    // Handle Infinite Scroll
    const onRowsRendered = ({ stopIndex }: { stopIndex: number }) => {
        if (hasNextPage && stopIndex >= rowCount - 5) {
            loadMore();
        }
    };

    const getRowHeight = useCallback(({ index }: { index: number }) => {
        const quest = quests[index]?.node;
        if (quest && expandedRows.has(quest.id)) {
            return 300; // Expanded height
        }
        return 80; // Base height
    }, [quests, expandedRows]);

    const rowRenderer = useCallback(({ index, key, style }: any) => {
        return (
            <QuestRow
                key={key}
                index={index}
                style={style}
                quests={quests}
                navigate={navigate}
                currentDate={currentDate}
                expandedRows={expandedRows}
                toggleRow={toggleRow}
            />
        );
    }, [quests, navigate, currentDate, expandedRows, toggleRow]);

    // Cast to any to avoid TS2786 errors with React 18
    const ListAny = List as any;
    const AutoSizerAny = AutoSizer as any;

    return (
        <PageWrapper currNavItem={PageNavItem.quests}>
            <div className="flex flex-col h-[calc(100vh-4rem)] gap-4">
                <QuestHeader />

                {/* Controls Section (Filters/Sorts) */}
                <div className="flex flex-row justify-between items-center bg-white p-4 rounded-md shadow-sm border border-gray-200">
                    <div className="flex-1">
                        <FilterControls type={FilterPageType.QUEST} />
                    </div>
                    <div className="ml-4">
                        <SortingControls type={PageNavItem.quests} />
                    </div>
                </div>

                {/* Table Container */}
                <div className="flex-1 flex flex-col bg-white rounded-md shadow overflow-hidden border border-gray-200">
                    {/* Header */}
                    <div className="flex flex-row bg-gray-50 border-b border-gray-200 shrink-0 pr-[15px]">
                        <div className={`${HEADER_CELL_CLASS} ${COL_WIDTHS.chevron}`}></div>
                        <div className={`${HEADER_CELL_CLASS} ${COL_WIDTHS.name}`}>Quest details</div>
                        <div className={`${HEADER_CELL_CLASS} ${COL_WIDTHS.updated}`}>Updated</div>
                        <div className={`${HEADER_CELL_CLASS} ${COL_WIDTHS.finished}`}>Finished</div>
                        <div className={`${HEADER_CELL_CLASS} ${COL_WIDTHS.output}`}>Output</div>
                        <div className={`${HEADER_CELL_CLASS} ${COL_WIDTHS.error}`}>Error</div>
                        <div className={`${HEADER_CELL_CLASS} ${COL_WIDTHS.creator}`}>Creator</div>
                    </div>

                    {/* Body */}
                    <div className="flex-1 min-h-0 h-full relative">
                        {loading && rowCount === 0 ? (
                            <div className="flex items-center justify-center h-full text-gray-500">
                                Loading...
                            </div>
                        ) : error ? (
                            <div className="flex items-center justify-center h-full text-red-500">
                                Error: {error.message}
                            </div>
                        ) : (
                            <AutoSizerAny>
                                {({ height, width }: { height: number, width: number }) => {
                                    if (height === 0 || width === 0) return null;
                                    return (
                                        <ListAny
                                            ref={listRef}
                                            height={height}
                                            width={width}
                                            rowCount={rowCount}
                                            rowHeight={getRowHeight}
                                            rowRenderer={rowRenderer}
                                            onRowsRendered={onRowsRendered}
                                            className="scrollbar-thin scrollbar-thumb-gray-300 scrollbar-track-transparent focus:outline-none"
                                            overscanRowCount={5}
                                        />
                                    );
                                }}
                            </AutoSizerAny>
                        )}
                    </div>

                    {/* Footer / Status */}
                    <div className="bg-gray-50 px-4 py-2 border-t border-gray-200 text-xs text-gray-500 flex justify-between shrink-0">
                         <div>Total: {data?.quests?.totalCount ?? 0}</div>
                         <div>{loading ? 'Refreshing...' : 'Updated'}</div>
                    </div>
                </div>
            </div>
        </PageWrapper>
    );
};

export default JulesQuests;
