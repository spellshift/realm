import { useCallback, useMemo } from "react";
import { CheckCircleIcon } from "@heroicons/react/24/solid";

import CodeBlock from "../../tavern-base-ui/CodeBlock";
import { VirtualizedTable } from "../../tavern-base-ui/virtualized-table/VirtualizedTable";
import { ExpandableConfig, VirtualizedTableColumn } from "../../tavern-base-ui/virtualized-table/types";
import { TomeNode, TomeDetailQueryResponse } from "../../../utils/interfacesQuery";
import { TomeTactic } from "../../../utils/enums";
import { safelyJsonParse } from "../../../utils/utils";
import { GET_TOME_DETAIL_QUERY } from "../../../utils/queries";
import { TomeTableProps } from "./types";
import { ParamLabelsDisplay } from "./ParamLabelsDisplay";

export const TomeTable = ({
    tomeIds,
    selectable = false,
    selectedTomeId,
    onSelectTome,
}: TomeTableProps) => {
    const handleItemClick = useCallback(
        (_tomeId: string, tome: TomeNode | null) => {
            if (!tome) return;
            if (selectable && onSelectTome) {
                onSelectTome(tome);
            }
        },
        [selectable, onSelectTome]
    );

    const getVariables = useCallback((id: string) => ({ id }), []);

    const extractData = useCallback(
        (response: TomeDetailQueryResponse): TomeNode | null => {
            return response?.tomes?.edges?.[0]?.node || null;
        },
        []
    );

    const expandable: ExpandableConfig<TomeNode> = useMemo(
        () => ({
            render: (tome) => (
                <div className="p-4">
                    {tome.description && (
                        <div className="line-clamp-2 break-all text-sm text-gray-600 mx-3">{tome.description}</div>
                    )}
                    <CodeBlock code={tome.eldritch} language="python" />
                </div>
            ),
            isExpandable: (tome) => !!tome.eldritch,
        }),
        []
    );

    const columns: VirtualizedTableColumn<TomeNode>[] = useMemo(() => {
        const baseColumns: VirtualizedTableColumn<TomeNode>[] = [
            {
                key: "name",
                label: "Name",
                width: "minmax(150px, 1fr)",
                render: (tome) => tome.name
            },
            {
                key: "parameters",
                label: "Parameters",
                width: "minmax(120px, 1fr)",
                render: (tome) => {
                    const { params: tomeParams } = safelyJsonParse(tome.paramDefs || "");
                    return <ParamLabelsDisplay params={tomeParams || []} />;
                },
            },
            {
                key: "tactic",
                label: "Tactic",
                width: "minmax(100px, 150px)",
                render: (tome) => {
                    const tacticLabel =
                        tome.tactic && tome.tactic !== "UNSPECIFIED"
                            ? TomeTactic[tome.tactic as keyof typeof TomeTactic]
                            : null;
                    return (
                        <div className="text-gray-600">
                            {tacticLabel || <span className="text-gray-400">-</span>}
                        </div>
                    );
                },
            },
        ];

        if (selectable) {
            baseColumns.push({
                key: "selection",
                label: "",
                width: "minmax(24px,30px)",
                render: (tome) => {
                    if (selectedTomeId === tome.id) {
                        return (
                            <div className="flex items-center justify-center h-full items-center text-purple-800">
                                <CheckCircleIcon className="w-6 h-6" />
                            </div>
                        );
                    }
                    return (
                        <div className="flex items-center justify-center h-full items-center text-purple-800">
                            <span
                                aria-hidden="true"
                                className="w-5 h-5 rounded-full border-2 border-black border-opacity-10"
                            />
                        </div>
                    );
                },
            });
        }

        return baseColumns;
    }, [selectable, selectedTomeId]);

    const rowSize = 80;
    const maxRows = 7;
    const maxHeight = "480px";
    const minHeight = tomeIds.length > maxRows
        ? maxHeight
        : `${(tomeIds.length * rowSize) + 20}px`;

    return (
        <VirtualizedTable<TomeNode, TomeDetailQueryResponse>
            items={tomeIds}
            columns={columns}
            query={GET_TOME_DETAIL_QUERY}
            getVariables={getVariables}
            extractData={extractData}
            expandable={expandable}
            onItemClick={handleItemClick}
            estimateRowSize={rowSize}
            height={maxHeight}
            minHeight={minHeight}
            minWidth="300px"
            headerVisible={true}
            growWithContent={true}
        />
    );
};

export default TomeTable;
