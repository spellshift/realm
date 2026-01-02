import { gql, useQuery, useMutation } from "@apollo/client";
import { GET_TOMES_QUERY } from "../../utils/queries";
import { BeaconNode, HostNode, TomeQueryTopLevel } from "../../utils/interfacesQuery";
import Button from "../tavern-base-ui/button/Button";
import { Play } from "lucide-react";
import { GET_QUEST_QUERY } from "../../utils/queries";
import { useFilters } from "../../context/FilterContext";
import { useState } from "react";
import AlertError from "../tavern-base-ui/AlertError";
import { useNavigate } from "react-router-dom";

interface QueueTomeButtonProps {
    tome: string;
    host?: HostNode;
    beacon?: BeaconNode;
}

const CREATE_QUEST_MUTATION = gql`
    mutation CreateQuest ($IDs: [ID!]!, $input: CreateQuestInput!) {
        createQuest(beaconIDs: $IDs, input: $input) {
            id
        }
    }
`;

export const QueueTomeButton = ({ tome: tomeName, host, beacon }: QueueTomeButtonProps) => {
    const { updateFilters } = useFilters();
    const [mutationError, setMutationError] = useState(false);
    const navigate = useNavigate();

    // Query for the tome by name
    const { data: tomeData, loading: tomeLoading, error: tomeError } = useQuery<TomeQueryTopLevel>(GET_TOMES_QUERY, {
        variables: { where: { name: tomeName } }
    });

    const [createQuest, { loading: mutationLoading }] = useMutation(CREATE_QUEST_MUTATION, {
        onCompleted: (result: { createQuest: { id: string } }) => {
            updateFilters({ 'filtersEnabled': false });
            navigate(`/tasks/${result.createQuest.id}`);
        },
        onError: () => {
            setMutationError(true);
        },
        refetchQueries: [GET_QUEST_QUERY]
    });

    // Find the target tome from the query result
    const targetTome = tomeData?.tomes?.edges?.[0]?.node;

    // Logic to select the best beacon
    const getTargetBeaconId = (): string | undefined => {
        if (beacon) return beacon.id;
        if (host && host.beacons?.edges) {
            let bestBeacon: BeaconNode | null = null;
            let minExpectedCallback = Infinity;

            for (const edge of host.beacons.edges) {
                const b = edge.node;
                const lastSeen = new Date(b.lastSeenAt).getTime();
                // interval is in seconds, convert to ms
                const expectedCallback = lastSeen + (b.interval * 1000);

                // We want the soonest expected callback (smallest value)
                if (expectedCallback < minExpectedCallback) {
                    minExpectedCallback = expectedCallback;
                    bestBeacon = b;
                }
            }
            return bestBeacon?.id;
        }
        return undefined;
    };

    const targetBeaconId = getTargetBeaconId();
    const isDisabled = !targetTome || !targetBeaconId || tomeLoading || !!tomeError;

    // Construct tooltip text
    let tooltipText = "";
    if (tomeLoading) tooltipText = "Loading tome...";
    else if (tomeError) tooltipText = "Error loading tome";
    else if (!targetTome) tooltipText = `Can't find tome: ${tomeName}`;
    else if (!targetBeaconId) tooltipText = "No suitable beacon found";
    else tooltipText = `Queue ${tomeName}`;

    const handleQueue = () => {
        if (!targetTome || !targetBeaconId) return;

        createQuest({
            variables: {
                IDs: [targetBeaconId],
                input: {
                    name: `Queue ${tomeName}`,
                    tomeID: targetTome.id,
                    parameters: "{}"
                }
            }
        });
    };

    if (mutationError) {
        return (
            <div className="relative group">
                <AlertError label="Error" details="Failed to queue tome" />
            </div>
        );
    }

    return (
        <div className="relative group inline-block" title={tooltipText}>
            <Button
                leftIcon={<Play className="h-4 w-4" />}
                onClick={handleQueue}
                disabled={isDisabled}
                isLoading={mutationLoading}
                buttonVariant="outline"
                buttonStyle={{ size: "sm", color: "purple" }}
            >
                Queue {tomeName}
            </Button>
            {/* Native title attribute used for tooltip as per standard HTML behavior,
                since a dedicated Tooltip component wasn't found in tavern-base-ui */}
        </div>
    );
};
