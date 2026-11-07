import { FC, useState } from "react";
import { useApolloClient, useMutation } from "@apollo/client";
import { Checkbox, useToast } from "@chakra-ui/react";
import { Popover, PopoverButton, PopoverPanel, Tab, TabGroup, TabList } from "@headlessui/react";
import { ChevronDown, Tags } from "lucide-react";
import Button from "../../../components/tavern-base-ui/button/Button";
import FormTextField from "../../../components/tavern-base-ui/FormTextField";
import { Table } from "../../../components/tavern-base-ui/table/Table";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import { useAuthorization } from "../../../context/AuthorizationContext";
import { GROUP_HOSTS_BY_SUBNET, RUN_HOST_AUTOGROUP } from "./queries";
import { useTagSummaryData } from "./useTagSummaryData";
import { useTagSummaryColumns } from "./useTagSummaryColumns";
import { TagKind } from "./types";

const TAB_KINDS: TagKind[] = ["group", "service"];
const TAB_LABELS = ["Group", "Service"];
type AutoTagAction = "groups" | "services" | "both" | "both-custom";

export const TagSummaryCard: FC = () => {
    const [selectedIndex, setSelectedIndex] = useState(0);
    const [serviceCountInput, setServiceCountInput] = useState("");
    const [includeGroups, setIncludeGroups] = useState(true);
    const [includeServices, setIncludeServices] = useState(true);
    const [busyAction, setBusyAction] = useState<AutoTagAction | null>(null);
    const tagKind = TAB_KINDS[selectedIndex];
    const { rows, hostCount, loading, error } = useTagSummaryData(tagKind);
    const autoTagDisabled = hostCount === 0 || busyAction !== null;
    const columns = useTagSummaryColumns();
    const { data: authorization } = useAuthorization();
    const client = useApolloClient();
    const toast = useToast();
    const [groupBySubnet] = useMutation(GROUP_HOSTS_BY_SUBNET);
    const [runHostAutogroup] = useMutation(RUN_HOST_AUTOGROUP);

    const applyAutoTags = async (action: AutoTagAction) => {
        if (hostCount === 0) return false;
        const hasCount = serviceCountInput.trim() !== "";
        const k = Number(serviceCountInput);
        if (action !== "both" && action !== "groups" && hasCount && (!Number.isInteger(k) || k < 1 || k > 2147483647)) {
            toast({ title: "Enter a positive whole number of service tags", status: "error", duration: 4000, isClosable: true });
            return false;
        }

        setBusyAction(action);
        let groupCount = 0;
        let serviceCount = 0;
        let succeeded = false;
        try {
            if (action === "groups" || action === "both" || action === "both-custom") {
                const result = await groupBySubnet();
                groupCount = result.data?.groupHostsBySubnet?.length ?? 0;
            }
            if (action !== "groups") {
                const result = await runHostAutogroup({ variables: { k: action === "both" || !hasCount ? null : k } });
                serviceCount = result.data?.runHostAutogroup?.length ?? 0;
            }
            toast({
                title: "Host auto tagging complete",
                description: `${groupCount} group tag(s) created or refreshed; ${serviceCount} service tag(s) created.`,
                status: "success",
                duration: 4000,
                isClosable: true,
            });
            succeeded = true;
        } catch (error) {
            toast({
                title: "Host auto tagging failed",
                description: error instanceof Error ? error.message : "The request failed.",
                status: "error",
                duration: 5000,
                isClosable: true,
            });
        } finally {
            if (groupCount > 0 || serviceCount > 0) {
                try {
                    await client.refetchQueries({ include: ["GetHostDetail", "GetHostContext", "GetAllHostsWithTagStats"] });
                } catch (error) {
                    toast({
                        title: "Tags updated, but the view could not refresh",
                        description: error instanceof Error ? error.message : "Reload the page to see the latest tags.",
                        status: "warning",
                        duration: 5000,
                        isClosable: true,
                    });
                }
            }
            setBusyAction(null);
        }
        return succeeded;
    };

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
                                                <div className={`rounded-md border p-3 ${includeGroups ? "border-purple-200 bg-purple-50" : "border-gray-200"}`}>
                                                    <Checkbox
                                                        isChecked={includeGroups}
                                                        onChange={(event) => setIncludeGroups(event.target.checked)}
                                                        colorScheme="purple"
                                                        isDisabled={busyAction !== null}
                                                    >
                                                        <span className="flex flex-col">
                                                            <span className="text-sm font-medium text-gray-900">Groups</span>
                                                            <span className="text-xs text-gray-500">Refresh group tags</span>
                                                        </span>
                                                    </Checkbox>
                                                </div>
                                                <div className={`rounded-md border p-3 ${includeServices ? "border-purple-200 bg-purple-50" : "border-gray-200"}`}>
                                                    <Checkbox
                                                        isChecked={includeServices}
                                                        onChange={(event) => setIncludeServices(event.target.checked)}
                                                        colorScheme="purple"
                                                        isDisabled={busyAction !== null}
                                                    >
                                                        <span className="flex flex-col">
                                                            <span className="text-sm font-medium text-gray-900">Services</span>
                                                            <span className="text-xs text-gray-500">Create service tags</span>
                                                        </span>
                                                    </Checkbox>
                                                </div>
                                            </div>
                                            {includeServices && (
                                                <FormTextField
                                                    htmlFor="autoTagServiceCount"
                                                    id="autoTagServiceCount"
                                                    label="Number of service tags (optional)"
                                                    type="number"
                                                    min={1}
                                                    step={1}
                                                    placeholder="Leave blank for automatic"
                                                    value={serviceCountInput}
                                                    onChange={(event) => setServiceCountInput(event.target.value)}
                                                />
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
