import { useCallback, useMemo, useState } from "react";
import { CheckCircleIcon } from "@heroicons/react/24/solid";

import CodeBlock from "../../tavern-base-ui/CodeBlock";
import { EmptyState, EmptyStateType } from "../../tavern-base-ui/EmptyState";
import FreeTextSearch from "../../tavern-base-ui/FreeTextSearch";
import { VirtualizedTable } from "../../tavern-base-ui/virtualized-table/VirtualizedTable";
import { ExpandableConfig, VirtualizedTableColumn } from "../../tavern-base-ui/virtualized-table/types";
import { TomeNode } from "../../../utils/interfacesQuery";
import { FilterBarOption, FieldInputParams } from "../../../utils/interfacesUI";
import { TomeTactic } from "../../../utils/enums";
import { safelyJsonParse } from "../../../utils/utils";
import { useTomes } from "./useTomes";
import { GET_TOME_DETAIL_QUERY } from "./queries";
import { TomeSelectionStepProps, TomeDetailQueryResponse } from "./types";
import { ConfigureParams } from "./ConfigureParams";
import { StepControls } from "./StepControls";
import { TomeFilterBar } from "../../TomeFilterBar";

const SelectionIndicator = ({ isSelected }: { isSelected: boolean }) => {
    if (isSelected) {
        return (
            <div className="shrink-0 text-purple-800">
                <CheckCircleIcon className="w-6 h-6" />
            </div>
        );
    }
    return (
        <span
            aria-hidden="true"
            className="w-5 h-5 rounded-full border-2 border-black border-opacity-10"
        />
    );
};

const ParamLabelsDisplay = ({ params }: { params: FieldInputParams[] }) => {
    if (!params || params.length === 0) return <span className="text-gray-400">-</span>;
    return (
        <div className="flex flex-row flex-wrap gap-1 text-sm">
            {params.map((element: FieldInputParams, index: number) => (
                <span key={`${index}_${element.name}`}>
                    {element.label || element.name}
                    {index < params.length - 1 && ","}
                </span>
            ))}
        </div>
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
        (_tomeId: string, tome: TomeNode | null) => {
            if (!tome) return;
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
                key: "name",
                label: "Name",
                width: "minmax(150px, 1fr)",
                render: (tome) => (
                    <div className="flex flex-col">
                        <div className="break-all text-base">
                            {tome.name}
                        </div>
                        {tome.description && (
                            <div className="line-clamp-2 break-all">
                                {tome.description}
                            </div>
                        )}
                    </div>
                ),
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
                    const tacticLabel = tome.tactic && tome.tactic !== "UNSPECIFIED"
                        ? TomeTactic[tome.tactic as keyof typeof TomeTactic]
                        : null;
                    return (
                        <div className="text-gray-600">
                            {tacticLabel || <span className="text-gray-400">-</span>}
                        </div>
                    );
                },
            },
            {
                key: "selection",
                label: "",
                width: "minmax(24px,30px)",
                render: (tome) => (
                    <div className="flex items-center justify-center h-full items-center">
                        <SelectionIndicator isSelected={selectedTomeId === tome.id} />
                    </div>
                ),
            },
        ],
        [selectedTomeId]
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
                        onItemClick={handleSelectTome}
                        estimateRowSize={60}
                        height="500px"
                        minHeight="200px"
                        minWidth="600px"
                        headerVisible={true}
                    />
                )}
            </div>

            {selectedTomeId && <ConfigureParams formik={formik} />}

            <StepControls formik={formik} setCurrStep={setCurrStep}/>
        </div>
    );
};

export default TomeSelectionStep;
