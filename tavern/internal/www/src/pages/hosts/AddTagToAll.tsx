import { useMemo, useState } from "react";
import { useApolloClient, useMutation } from "@apollo/client";
import { useToast } from "@chakra-ui/react";
import { TagIcon } from "@heroicons/react/24/outline";
import Select, { SingleValue, createFilter } from "react-select";
import { ButtonDialogPopover } from "../../components/ButtonDialogPopover";
import Button from "../../components/tavern-base-ui/button/Button";
import { useFilters } from "../../context/FilterContext";
import { useSorts } from "../../context/SortContext";
import { PageNavItem } from "../../utils/enums";
import { FilterBarOption, KindOfTag } from "../../utils/interfacesUI";
import { useTagOptions } from "../host-details/components/editable-tag/useTagOptions";
import { GET_HOST_IDS_QUERY, UPDATE_TAG_HOSTS_MUTATION } from "./queries";
import { HostIdsQueryTopLevel } from "./types";
import { getHostIdsQuery } from "./useHostIds";

interface AddTagToAllProps {
    totalCount?: number;
    onComplete?: () => void;
}

const BULK_FETCH_PAGE_SIZE = 100;

export const AddTagToAll = ({ totalCount, onComplete }: AddTagToAllProps) => {
    const toast = useToast();
    const client = useApolloClient();
    const { filters } = useFilters();
    const { sorts } = useSorts();
    const hostSort = sorts[PageNavItem.hosts];

    const [selectedTag, setSelectedTag] = useState<FilterBarOption | null>(null);
    const [isApplying, setIsApplying] = useState(false);

    const { options: groupTagOptions, isLoading: groupTagsLoading } = useTagOptions("group");
    const { options: serviceTagOptions, isLoading: serviceTagsLoading } = useTagOptions("service");

    const groupedOptions = useMemo(
        () => [
            { label: "Group", options: groupTagOptions },
            { label: "Service", options: serviceTagOptions },
        ],
        [groupTagOptions, serviceTagOptions]
    );

    const optionsLoading = groupTagsLoading || serviceTagsLoading;

    const [updateTagMutation] = useMutation(UPDATE_TAG_HOSTS_MUTATION);

    const handleAddTagToAll = async () => {
        if (!selectedTag) {
            return;
        }

        setIsApplying(true);
        try {
            // Fetch every host id matching the current filter (paginate through all pages)
            const allHostIds: string[] = [];
            let after: string | undefined = undefined;
            let hasNextPage = true;

            while (hasNextPage) {
                const variables = getHostIdsQuery(filters, after as never, hostSort, BULK_FETCH_PAGE_SIZE, selectedTag.kind as KindOfTag);
                const result = await client.query<HostIdsQueryTopLevel>({
                    query: GET_HOST_IDS_QUERY,
                    variables,
                    fetchPolicy: "network-only",
                });

                const edges = result.data?.hosts?.edges ?? [];
                for (const edge of edges) {
                    if (edge?.node?.id) {
                        allHostIds.push(edge.node.id);
                    }
                }

                hasNextPage = result.data?.hosts?.pageInfo?.hasNextPage ?? false;
                after = result.data?.hosts?.pageInfo?.endCursor ?? undefined;

                if (!after && hasNextPage) {
                    break;
                }
            }

            const updatedCount = allHostIds.length;
            const skippedCount = Math.max((totalCount ?? updatedCount) - updatedCount, 0);

            if (updatedCount === 0) {
                toast({
                    title: "No hosts to update",
                    description: (totalCount ?? 0) > 0
                        ? `All ${totalCount} hosts in the current view already have a ${selectedTag.kind} tag.`
                        : "There are no hosts in the current view.",
                    status: "info",
                    duration: 3000,
                    isClosable: true,
                    position: "bottom-right",
                });
                return;
            }

            await updateTagMutation({
                variables: {
                    tagID: selectedTag.id,
                    input: { addHostIDs: allHostIds },
                },
            });

            toast({
                title: "Success",
                description: `Tag "${selectedTag.name}" added to ${updatedCount} host${updatedCount === 1 ? "" : "s"} (${skippedCount} skipped, already have a ${selectedTag.kind} tag).`,
                status: "success",
                duration: 3000,
                isClosable: true,
                position: "bottom-right",
            });

            setSelectedTag(null);
            onComplete?.();
        } catch (error) {
            toast({
                title: "Error adding tag",
                description: `${error}`,
                status: "error",
                duration: 3000,
                isClosable: true,
                position: "bottom-right",
            });
        } finally {
            setIsApplying(false);
        }
    };

    const noHosts = totalCount === 0;

    return (
        <ButtonDialogPopover label="Add Tag to All" leftIcon={<TagIcon className="w-4" />}>
            <div className="flex flex-col gap-3">
                <div className="flex flex-row justify-between pb-2 border-gray-100 border-b-2 items-center">
                    <h3 className="font-medium text-lg text-gray-700">Add Tag to All</h3>
                </div>
                <p className="text-sm text-gray-600">
                    {noHosts
                        ? "There are no hosts in the current view."
                        : `Apply a Group or Service tag to all ${totalCount ?? ""} host${totalCount === 1 ? "" : "s"} in the current view${totalCount !== undefined ? "" : " matching the current filters"}. Hosts that already have a tag of that kind will be skipped.`}
                </p>
                <div className="flex flex-col gap-1">
                    <label className="font-medium text-gray-700">Tag</label>
                    <Select
                        isClearable
                        isSearchable
                        isLoading={optionsLoading}
                        isDisabled={isApplying || optionsLoading}
                        options={groupedOptions}
                        value={selectedTag}
                        onChange={(newValue: SingleValue<FilterBarOption>) => setSelectedTag(newValue)}
                        filterOption={createFilter({
                            matchFrom: "any",
                            stringify: (option) => `${option.label}`,
                        })}
                        placeholder="Select a Group or Service tag"
                        aria-label="Select a Group or Service tag"
                    />
                </div>
                <div className="flex justify-end">
                    <Button
                        buttonVariant="solid"
                        buttonStyle={{ color: "purple", size: "md" }}
                        onClick={handleAddTagToAll}
                        disabled={!selectedTag || isApplying || noHosts}
                        isLoading={isApplying}
                    >
                        Add tag to all
                    </Button>
                </div>
            </div>
        </ButtonDialogPopover>
    );
};

export default AddTagToAll;
