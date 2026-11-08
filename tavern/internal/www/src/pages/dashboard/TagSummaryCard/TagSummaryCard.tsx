import { FC, useState } from "react";
import { Checkbox, Description, Field, Input, Label, Popover, PopoverButton, PopoverPanel, Tab, TabGroup, TabList } from "@headlessui/react";
import { Check, ChevronDown, Tags } from "lucide-react";
import Button from "../../../components/tavern-base-ui/button/Button";
import { Table } from "../../../components/tavern-base-ui/table/Table";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import { useAuthorization } from "../../../context/AuthorizationContext";
import { useTagSummaryData } from "./useTagSummaryData";
import { useTagSummaryColumns } from "./useTagSummaryColumns";
import { useAutoTagHosts, AutoTagAction } from "./useAutoTagHosts";
import { TagKind } from "./types";

const TAB_KINDS: TagKind[] = ["group", "service"];
const TAB_LABELS = ["Group", "Service"];

export const TagSummaryCard: FC = () => {
    const [selectedIndex, setSelectedIndex] = useState(0);
    const tagKind = TAB_KINDS[selectedIndex];
    const { rows, hostCount, loading, error } = useTagSummaryData(tagKind);
    const columns = useTagSummaryColumns();
    const { data: authorization } = useAuthorization();
    const {
        applyAutoTags,
        busyAction,
        autoTagDisabled,
        includeGroups,
        setIncludeGroups,
        includeServices,
        setIncludeServices,
        serviceCountInput,
        setServiceCountInput,
    } = useAutoTagHosts(hostCount);

    return (
        <div className="bg-white rounded-lg border border-gray-200 py-2 px-6 flex flex-col gap-4">
            <div className="flex flex-row justify-between items-center">
                <h3 className="text-lg font-semibold text-gray-900">Access by Tag</h3>

                <TabGroup selectedIndex={selectedIndex} onChange={setSelectedIndex}>
                    <TabList className="flex rounded-lg bg-gray-100 p-1">
                        {TAB_LABELS.map((label) => (
                            <Tab
                                key={label}
                                className={({ selected }) =>
                                    `px-4 py-1.5 text-sm font-medium rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-purple-600 focus:ring-offset-2 ${selected
                                        ? "bg-white text-purple-800 semi-bold shadow-sm"
                                        : "text-gray-600 hover:text-gray-900 hover:bg-gray-50"
                                    }`
                                }
                            >
                                {label}
                            </Tab>
                        ))}
                    </TabList>
                </TabGroup>
            </div>

            {loading && rows.length === 0 ? (
                <EmptyState type={EmptyStateType.loading} label="Loading tags..." />
            ) : error ? (
                <EmptyState type={EmptyStateType.error} label="Failed to load tag data" details={error.message} />
            ) : rows.length === 0 ? (
                <div className="min-h-72">
                    <EmptyState
                        type={EmptyStateType.noData}
                        label={hostCount === 0 ? "No hosts found" : `No ${tagKind} tags found`}
                        details={authorization?.me?.isAdmin
                            ? (hostCount === 0 ? "Add a host before auto tagging." : "Organize hosts into groups and services.")
                            : undefined}
                    >
                        {authorization?.me?.isAdmin && (
                            <Popover className="relative flex items-center justify-center">
                                <Button
                                    leftIcon={<Tags className="w-5 h-5" />}
                                    buttonStyle={{ color: "purple", size: "md" }}
                                    className="rounded-r-none"
                                    disabled={autoTagDisabled}
                                    isLoading={busyAction === "both"}
                                    onClick={() => applyAutoTags("both")}
                                >
                                    Auto tag all
                                </Button>
                                <PopoverButton
                                    as={Button}
                                    aria-label="Advanced auto tag options"
                                    buttonStyle={{ color: "purple", size: "md", xPadding: "xs" }}
                                    className="rounded-l-none border-l border-purple-400"
                                    disabled={autoTagDisabled}
                                    leftIcon={<ChevronDown className="w-5 h-5" />}
                                />
                                <PopoverPanel className="absolute bottom-full left-1/2 z-50 mb-2 w-80 -translate-x-1/2 rounded-lg border border-gray-200 bg-white p-4 shadow-lg">
                                    {({ close }) => (
                                        <div className="flex flex-col gap-3">
                                            <div>
                                                <p className="text-sm font-semibold text-gray-900">Auto tagging options</p>
                                                <p className="text-xs text-gray-500">Choose which tags to apply.</p>
                                            </div>
                                            <div className="flex flex-col gap-2">
                                                <Field className={`flex items-center gap-3 rounded-md border p-3 ${includeGroups ? "border-purple-200 bg-purple-50" : "border-gray-200"}`}>
                                                    <Checkbox
                                                        checked={includeGroups}
                                                        onChange={setIncludeGroups}
                                                        disabled={busyAction !== null}
                                                        className={`flex h-4 w-4 shrink-0 items-center justify-center rounded border focus:outline-none focus:ring-2 focus:ring-purple-600 focus:ring-offset-2 ${includeGroups ? "border-purple-600 bg-purple-600" : "border-gray-400 bg-white"}`}
                                                    >
                                                        {includeGroups && <Check className="h-3 w-3 text-white" strokeWidth={3} />}
                                                    </Checkbox>
                                                    <div className="flex flex-col">
                                                        <Label className="text-sm font-medium text-gray-900">Groups</Label>
                                                        <Description className="text-xs text-gray-500">Refresh group tags</Description>
                                                    </div>
                                                </Field>
                                                <Field className={`flex items-center gap-3 rounded-md border p-3 ${includeServices ? "border-purple-200 bg-purple-50" : "border-gray-200"}`}>
                                                    <Checkbox
                                                        checked={includeServices}
                                                        onChange={setIncludeServices}
                                                        disabled={busyAction !== null}
                                                        className={`flex h-4 w-4 shrink-0 items-center justify-center rounded border focus:outline-none focus:ring-2 focus:ring-purple-600 focus:ring-offset-2 ${includeServices ? "border-purple-600 bg-purple-600" : "border-gray-400 bg-white"}`}
                                                    >
                                                        {includeServices && <Check className="h-3 w-3 text-white" strokeWidth={3} />}
                                                    </Checkbox>
                                                    <div className="flex flex-col">
                                                        <Label className="text-sm font-medium text-gray-900">Services</Label>
                                                        <Description className="text-xs text-gray-500">Create service tags</Description>
                                                    </div>
                                                </Field>
                                            </div>
                                            {includeServices && (
                                                <Field className="flex flex-col gap-1">
                                                    <Label className="text-sm font-medium text-gray-900">Number of service tags (optional)</Label>
                                                    <Input
                                                        id="autoTagServiceCount"
                                                        type="number"
                                                        min={1}
                                                        step={1}
                                                        placeholder="Leave blank for automatic"
                                                        value={serviceCountInput}
                                                        onChange={(event) => setServiceCountInput(event.target.value)}
                                                        className="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 placeholder:text-gray-400 focus:outline-none focus:ring-2 focus:ring-purple-600"
                                                    />
                                                </Field>
                                            )}
                                            <div className="flex justify-end">
                                                <Button
                                                    buttonStyle={{ color: "purple", size: "sm" }}
                                                    disabled={autoTagDisabled || (!includeGroups && !includeServices)}
                                                    isLoading={busyAction !== null}
                                                    onClick={async () => {
                                                        const action: AutoTagAction = includeGroups
                                                            ? (includeServices ? "both-custom" : "groups")
                                                            : "services";
                                                        if (await applyAutoTags(action)) close();
                                                    }}
                                                >
                                                    Run selected
                                                </Button>
                                            </div>
                                        </div>
                                    )}
                                </PopoverPanel>
                            </Popover>
                        )}
                    </EmptyState>
                </div>
            ) : (
                <div className="h-72 w-full overflow-x-scroll">
                    <Table columns={columns} data={rows} initialSorting={[{ id: "tagName", desc: false }]} />
                </div>
            )}
        </div>
    );
};

export default TagSummaryCard;
