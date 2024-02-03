import { ApolloError } from "@apollo/client";
import { Accordion, AccordionButton, AccordionIcon, AccordionItem, AccordionPanel, Box } from "@chakra-ui/react";
import EmptyStateNoBeacon from "../../../components/empty-states/EmptyStateNoBeacon";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import { HostType } from "../../../utils/consts";
import BeaconTable from "./BeaconTable";

type Props = {
    loading: boolean;
    error: ApolloError | undefined;
    host: HostType | null;
}
const BeaconAccordion = (props: Props) => {
    const { loading, error, host } = props;
    console.log(host);
    return (
        <Accordion allowToggle className='w-full'>
            <AccordionItem>
                <AccordionButton>
                    <Box as="span" flex='1' textAlign='left'>
                        <h2 className="text-2xl font-semibold text-gray-900">Beacons</h2>
                    </Box>
                    <AccordionIcon />
                </AccordionButton>
                <AccordionPanel>
                    {loading ? (
                        <EmptyState type={EmptyStateType.loading} label="Loading beacons..." />
                    ) : error ? (
                        <EmptyState type={EmptyStateType.error} label="Error loading beacons..." />
                    ) : (
                        <div>
                            {host?.beacons && host?.beacons?.length > 0 ? (
                                <BeaconTable beacons={host.beacons} />
                            )
                                : (
                                    <EmptyStateNoBeacon />
                                )}
                        </div>
                    )}
                </AccordionPanel>
            </AccordionItem>
        </Accordion>
    );
}
export default BeaconAccordion;
