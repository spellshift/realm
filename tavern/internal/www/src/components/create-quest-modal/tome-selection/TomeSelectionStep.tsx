import { useCallback, useMemo, useState } from "react";
import { CheckCircleIcon } from "@heroicons/react/24/solid";

import CodeBlock from "../../tavern-base-ui/CodeBlock";
import { EmptyState, EmptyStateType } from "../../tavern-base-ui/EmptyState";
import FreeTextSearch from "../../tavern-base-ui/FreeTextSearch";
import { VirtualizedTable } from "../../tavern-base-ui/virtualized-table/VirtualizedTable";
import { ExpandableConfig, VirtualizedTableColumn } from "../../tavern-base-ui/virtualized-table/types";
import { TomeNode } from "../../../utils/interfacesQuery";
import { FilterBarOption } from "../../../utils/interfacesUI";
import { safelyJsonParse } from "../../../utils/utils";
import { useTomes } from "./useTomes";
import { GET_TOME_DETAIL_QUERY } from "./queries";
import { TomeSelectionStepProps, TomeDetailQueryResponse } from "./types";
import TomeDetails from "../../TomeComponents";
import { ConfigureParams } from "./ConfigureParams";
import { StepControls } from "./StepControls";
import { TomeFilterBar } from "../../TomeFilterBar";

const SelectionIndicator = ({ isSelected }: {isSelected: boolean}) => {
    if (isSelected) {
        return (
            <div className="shrink-0 text-purple-800 mt-2">
                <CheckCircleIcon className="w-7" />
            </div>
        );
    }
    return (
        <span
            aria-hidden="true"
            className="w-5 h-5 rounded-full border-2 border-black border-opacity-10 mt-2"
        />
    );
};

export const TomeSelectionStep = ({ setCurrStep, formik }: TomeSelectionStepProps) => {
    const selectedTomeId = formik.values.tomeId;
    const [tomeMultiSearch, setTomeMultiSearch] = useState("");
    const [tomeFields, setTomeFields] = useState<FilterBarOption[]>([]);

    const {
        tomeIds,
        initialLoading,
    } = useTomes(tomeFields, tomeMultiSearch);

    const handleSelectTome = useCallback(
        (tome: TomeNode) => {
            const { params: tomeParams } = safelyJsonParse(tome.paramDefs || "");
            formik.setFieldValue("tomeId", tome.id);
            formik.setFieldValue("params", tomeParams || []);
        },
        [formik]
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
                    <CodeBlock code={tome.eldritch} language="python" />
                </div>
            ),
            isExpandable: (tome) => !!tome.eldritch,
        }),
        []
    );

    const columns: VirtualizedTableColumn<TomeNode>[] = useMemo(
        () => [
            {
                key: "tome-details",
                label: "Tomes",
                width: "minmax(300px, 1fr)",
                render: (tome) => {
                    const isSelected = selectedTomeId === tome.id;
                    const { params: tomeParams } = safelyJsonParse(tome.paramDefs || "");

                    return (
                        <div
                            className="flex flex-row gap-2 p-2 cursor-pointer items-center justify-between"
                            onClick={() => handleSelectTome(tome)}
                            role="button"
                            tabIndex={0}
                            onKeyDown={(e) => {
                                if (e.key === "Enter" || e.key === " ") {
                                    e.preventDefault();
                                    handleSelectTome(tome);
                                }
                            }}
                            aria-pressed={isSelected}
                            aria-label={`Select tome ${tome.name}`}
                        >
                            <TomeDetails tome={tome} params={tomeParams} showParamValues={false}/>
                            <SelectionIndicator isSelected={isSelected} />
                        </div>
                    );
                },
                renderSkeleton: () => (
                    <div className="flex flex-col min-w-0 space-y-2 p-3">
                        <div className="h-5 bg-gray-200 rounded animate-pulse w-3/4"></div>
                        <div className="h-4 bg-gray-200 rounded animate-pulse w-full"></div>
                        <div className="h-3 bg-gray-200 rounded animate-pulse w-1/2"></div>
                    </div>
                ),
            },
        ],
        [selectedTomeId, handleSelectTome]
    );

    const hasActiveFilters = tomeFields.length > 0 || tomeMultiSearch;

    if (initialLoading) {
        return (
            <div className="flex flex-col gap-6">
                <div className="flex flex-col gap-1">
                    <h2 className="text-xl font-semibold text-gray-900">Select a tome</h2>
                    <p className="text-sm text-gray-700 italic">
                        Choose a tome to execute on selected beacons
                    </p>
                </div>
                <EmptyState type={EmptyStateType.loading} label="Loading tomes..." />
            </div>
        );
    }

    return (
        <div className="flex flex-col gap-6">
            <div className="flex flex-col gap-1">
                <h2 className="text-xl font-semibold text-gray-900">Select a tome</h2>
                <p className="text-sm text-gray-700 italic">
                    Choose a tome to execute on selected beacons
                </p>
            </div>
            <div className="flex flex-col gap-2">
                <div className="grid grid-cols-2 gap-2">
                    <TomeFilterBar
                        setFiltersSelected={setTomeFields}
                        filtersSelected={tomeFields}
                    />
                    <FreeTextSearch
                        placeholder="Tome name description & params"
                        setSearch={setTomeMultiSearch}
                    />
                </div>
                {tomeIds.length === 0 ? (
                    <div className="flex items-center justify-center py-8 text-gray-500 h-[300px]">
                        {hasActiveFilters
                            ? "No tomes matching your search."
                            : "No tomes available."}
                    </div>
                ) : (
                    <VirtualizedTable<TomeNode, TomeDetailQueryResponse>
                        items={tomeIds}
                        columns={columns}
                        query={GET_TOME_DETAIL_QUERY}
                        getVariables={getVariables}
                        extractData={extractData}
                        expandable={expandable}
                        estimateRowSize={120}
                        height="500px"
                        minHeight="200px"
                        minWidth="400px"
                        headerVisible={false}
                    />
                )}
            </div>

            {selectedTomeId && <ConfigureParams formik={formik} />}

            <StepControls formik={formik} setCurrStep={setCurrStep}/>
        </div>
    );
};

export default TomeSelectionStep;
