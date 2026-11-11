import { FC, useState } from "react";
import { Checkbox, Description, Field, Input, Label, Tab, TabGroup, TabList } from "@headlessui/react";
import { Check, Tags } from "lucide-react";
import Button from "../../../components/tavern-base-ui/button/Button";
import Modal from "../../../components/tavern-base-ui/Modal";
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
    const [autoTagOpen, setAutoTagOpen] = useState(false);
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
                            <Button
                                leftIcon={<Tags className="w-5 h-5" />}
                                buttonStyle={{ color: "purple", size: "md" }}
                                disabled={autoTagDisabled}
                                onClick={() => setAutoTagOpen(true)}
                            >
                                Auto tag
                            </Button>
                        )}
                    </EmptyState>
                </div>
            ) : (
                <div className="h-72 w-full overflow-x-scroll">
                    <Table columns={columns} data={rows} initialSorting={[{ id: "tagName", desc: false }]} />
                </div>
            )}
            {authorization?.me?.isAdmin && (
                <Modal isOpen={autoTagOpen} setOpen={setAutoTagOpen} size="sm">
                    <div className="flex h-full flex-col gap-6">
                        <div>
                            <h2 className="text-lg font-semibold text-gray-900">Auto tagging options</h2>
                            <p className="mt-1 text-sm text-gray-600">Choose which tags to apply to your hosts.</p>
                        </div>
                        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                            <Field className={`relative rounded-md border ${includeGroups ? "border-purple-200 bg-purple-50" : "border-gray-200 bg-white"}`}>
                                <Checkbox
                                    checked={includeGroups}
                                    onChange={setIncludeGroups}
                                    disabled={busyAction !== null}
                                    className={`absolute right-4 top-4 z-10 flex h-4 w-4 items-center justify-center rounded border focus:outline-none focus:ring-2 focus:ring-purple-600 focus:ring-offset-2 ${includeGroups ? "border-purple-600 bg-purple-600" : "border-gray-400 bg-white"}`}
                                >
                                    {includeGroups && <Check className="h-3 w-3 text-white" strokeWidth={3} />}
                                </Checkbox>
                                <Label className="flex h-full cursor-pointer flex-col gap-2 p-3">
                                    <span className="flex h-16 items-center justify-center rounded-md border border-purple-100 bg-white">
                                        <svg viewBox="0 0 120 45" fill="none" stroke="currentColor" strokeWidth="1.6" className="h-10 w-28 text-purple-600" aria-hidden="true">
                                            <path d="M15 22h28m13 0h26m12 0h12" />
                                            <circle cx="12" cy="22" r="7" fill="white" />
                                            <circle cx="50" cy="22" r="7" fill="white" />
                                            <circle cx="89" cy="22" r="7" fill="white" />
                                            <circle cx="110" cy="22" r="7" fill="white" />
                                        </svg>
                                    </span>
                                    <span className="text-sm font-medium text-gray-900">Group by subnet</span>
                                    <Description as="span" className="text-xs text-gray-500">Hosts on the same reported subnet get one group tag.</Description>
                                </Label>
                            </Field>
                            <Field className={`relative rounded-md border ${includeServices ? "border-purple-200 bg-purple-50" : "border-gray-200 bg-white"}`}>
                                <Checkbox
                                    checked={includeServices}
                                    onChange={setIncludeServices}
                                    disabled={busyAction !== null}
                                    className={`absolute right-4 top-4 z-10 flex h-4 w-4 items-center justify-center rounded border focus:outline-none focus:ring-2 focus:ring-purple-600 focus:ring-offset-2 ${includeServices ? "border-purple-600 bg-purple-600" : "border-gray-400 bg-white"}`}
                                >
                                    {includeServices && <Check className="h-3 w-3 text-white" strokeWidth={3} />}
                                </Checkbox>
                                <Label className="flex h-full cursor-pointer flex-col gap-2 p-3">
                                    <span className="flex h-16 items-center justify-center rounded-md border border-purple-100 bg-white">
                                        <svg viewBox="0 0 120 45" fill="none" stroke="currentColor" strokeWidth="1.6" className="h-10 w-28 text-purple-600" aria-hidden="true">
                                            <rect x="5" y="8" width="29" height="29" rx="5" fill="white" />
                                            <rect x="45" y="8" width="29" height="29" rx="5" fill="white" />
                                            <rect x="85" y="8" width="29" height="29" rx="5" fill="white" />
                                            <path d="M13 16h13M13 22h10M53 16h13M53 22h10M93 16h13M93 22h10" />
                                        </svg>
                                    </span>
                                    <span className="text-sm font-medium text-gray-900">Cluster by processes</span>
                                    <Description as="span" className="text-xs text-gray-500">Process list data creates service tags for similar hosts.</Description>
                                </Label>
                            </Field>
                        </div>
                        <div>
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
                        </div>
                        <div className="mt-auto flex justify-end gap-2">
                            <Button buttonVariant="ghost" buttonStyle={{ color: "gray", size: "md" }} disabled={busyAction !== null} onClick={() => setAutoTagOpen(false)}>
                                Cancel
                            </Button>
                            <Button
                                buttonStyle={{ color: "purple", size: "md" }}
                                disabled={autoTagDisabled || (!includeGroups && !includeServices)}
                                isLoading={busyAction !== null}
                                onClick={async () => {
                                    const action: AutoTagAction = includeGroups
                                        ? (includeServices ? "both" : "groups")
                                        : "services";
                                    if (await applyAutoTags(action)) setAutoTagOpen(false);
                                }}
                            >
                                Confirm
                            </Button>
                        </div>
                    </div>
                </Modal>
            )}
        </div>
    );
};

export default TagSummaryCard;
