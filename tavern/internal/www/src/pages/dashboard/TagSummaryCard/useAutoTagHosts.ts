import { useState } from "react";
import { useApolloClient, useMutation } from "@apollo/client";
import { useToast } from "@chakra-ui/react";
import { GROUP_HOSTS_BY_SUBNET, RUN_HOST_AUTOGROUP } from "./queries";

export type AutoTagAction = "groups" | "services" | "both";

export const useAutoTagHosts = (hostCount: number) => {
    const [serviceCountInput, setServiceCountInput] = useState("");
    const [includeGroups, setIncludeGroups] = useState(true);
    const [includeServices, setIncludeServices] = useState(true);
    const [busyAction, setBusyAction] = useState<AutoTagAction | null>(null);
    const autoTagDisabled = hostCount === 0 || busyAction !== null;

    const client = useApolloClient();
    const toast = useToast();
    const [groupBySubnet] = useMutation(GROUP_HOSTS_BY_SUBNET);
    const [runHostAutogroup] = useMutation(RUN_HOST_AUTOGROUP);

    const applyAutoTags = async (action: AutoTagAction) => {
        if (hostCount === 0) return false;
        const hasCount = serviceCountInput.trim() !== "";
        const k = Number(serviceCountInput);
        if (action !== "groups" && hasCount && (!Number.isInteger(k) || k < 1 || k > 2147483647)) {
            toast({ title: "Enter a positive whole number of service tags", status: "error", duration: 4000, isClosable: true });
            return false;
        }

        setBusyAction(action);
        let groupCount = 0;
        let serviceCount = 0;
        let succeeded = false;
        try {
            if (action === "groups" || action === "both") {
                const result = await groupBySubnet();
                groupCount = result.data?.groupHostsBySubnet?.length ?? 0;
            }
            if (action !== "groups") {
                const result = await runHostAutogroup({ variables: { k: hasCount ? k : null } });
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

    return {
        applyAutoTags,
        busyAction,
        autoTagDisabled,
        includeGroups,
        setIncludeGroups,
        includeServices,
        setIncludeServices,
        serviceCountInput,
        setServiceCountInput,
    };
};
