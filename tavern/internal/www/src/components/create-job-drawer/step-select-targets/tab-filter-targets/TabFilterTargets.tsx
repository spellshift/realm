import { Heading, Text, Stack, StackItem, TabPanel, Icon, Box, Button, Card, CardBody, Checkbox, Flex } from "@chakra-ui/react";
import Select from "react-select"
import { MdInfoOutline } from "react-icons/md";
import { SelectOption, SelectOptionGroup } from "../../../utils/consts";
import { useState } from "react";
import { SearchFilter } from "./search-filter";

type TabFilterTargetsParams = {
    beacons: Array<any>;
    targets: any;
    setFieldValue: (arg1: any, arg2: any) => void;
}
export const TabFilterTargets = (props: TabFilterTargetsParams) => {
    const {beacons, targets, setFieldValue} = props;

    const [filteredBeacons, setFilteredBeacons] = useState(beacons);

    const toggleCheck = (inputName :any) => {
        const newState = {...targets};
        newState[inputName] = !targets[inputName];
        setFieldValue('beacons', newState);
    };

    const handleCheckAllFiltered = () => {
        const newState = { ...targets };
        filteredBeacons.map((beacon) => {
            newState[beacon.id] = true;
        });
        setFieldValue('beacons', newState);
    }

    return (
        <TabPanel >
            <Stack direction="column" gap="2">
                <StackItem>
                    <Heading size="sm" mb={2}> Use the dropdown to filter the list then select targets</Heading>
                    <SearchFilter beacons={beacons} setFilteredBeacons={setFilteredBeacons} />
                </StackItem>
                <StackItem>
                    <Box p={2} className="md-scroll-container" borderRadius={"md"}>
                        <Stack direction="column" gap={2}>
                            <StackItem>
                                <Button size={"sm"} onClick={()=> handleCheckAllFiltered()}>Select all options below</Button>
                            </StackItem>
                            {filteredBeacons.map((beacon: any) => {
                                // TODO change to map to avoid extra loop
                                let group = (beacon?.tags).find( (obj : any) => {
                                    return obj?.kind === "group"
                                });
                                let service = (beacon?.tags).find( (obj : any) => {
                                    return obj?.kind === "service"
                                });

                                return (
                                    <StackItem key={beacon?.id}>
                                        <Card>
                                            <CardBody>
                                                <Checkbox colorScheme={"purple"} size="lg" isChecked={targets[beacon.id]} onChange={()=> toggleCheck(beacon.id)}>
                                                    <Stack ml={4} w={"xl"}>
                                                        <StackItem>
                                                                <Text fontSize={"md"}>{beacon.hostname}</Text>
                                                        </StackItem>
                                                        <StackItem>
                                                            <Flex direction="row" wrap={"wrap"}>
                                                                <Text fontSize={"sm"}>
                                                                    {group?.name} | {service?.name} | {beacon.principal}
                                                                </Text>
                                                            </Flex>
                                                        </StackItem>
                                                    </Stack>
                                                </Checkbox>
                                            </CardBody>
                                        </Card>
                                    </StackItem>
                                );
                            })}
                            {filteredBeacons.length === 0 &&
                                <Text fontSize={"sm"} p={2}>Try adjusting filter. No results found.</Text>
                            }
                        </Stack>
                    </Box>
                </StackItem>
            </Stack>
        </TabPanel>
    );
}