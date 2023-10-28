import React from "react";
import { Text, Stack, StackItem, Card, CardBody, Checkbox, Flex, Badge } from "@chakra-ui/react";

export function areEqual(prevProps: any, nextProps: any) {
    const beacon = prevProps.beaconsToDisplay[prevProps.index];
    const nextBeacon = nextProps.beaconsToDisplay[nextProps.index];
    return prevProps.beaconsToDisplay === nextProps.beaconsToDisplay && prevProps.beaconsSelected[beacon.id] === nextProps.beaconsSelected[nextBeacon.id];
}

type Props = {
    index: number;
    style: any;
    beaconsToDisplay: Array<any>;
    toggleCheck: (arg: any) => void;
    beaconsSelected: any;
};

export const BeaconOption = (props: Props) => {
    const {index, style, beaconsToDisplay, toggleCheck, beaconsSelected } = props;
    // Your card component goes here
    const beacon = beaconsToDisplay[index];
    const group = (beacon?.host?.tags).find( (obj : any) => {
        return obj?.kind === "group"
    });
    const service = (beacon?.host?.tags).find( (obj : any) => {
        return obj?.kind === "service"
    });
    const isChecked = beaconsSelected[beacon.id];

    return (
        <div style={style} key={`beacon_option_${beacon.id}`}>
            <Card>
                <CardBody>
                    <Checkbox colorScheme={"purple"} size="lg" isChecked={isChecked} onChange={()=> toggleCheck(beacon.id)}>
                        <Stack ml={4} w={"xl"}>
                            <StackItem>
                                    <Text fontSize={"md"}>{beacon.name}</Text>
                            </StackItem>
                            <StackItem>
                                <Flex direction="row" wrap={"wrap"} gap={2}>
                                    {group?.name && <Badge>{group?.name}</Badge>}
                                    {service?.name && <Badge>{service?.name}</Badge>}
                                    {beacon?.host?.primaryIP && <Badge>{beacon?.host?.primaryIP}</Badge>}
                                    {beacon?.principal && <Badge>{beacon?.principal}</Badge>}
                                </Flex>
                            </StackItem>
                        </Stack>
                    </Checkbox>
                </CardBody>
            </Card>
        </div>
    );
};

export default React.memo(BeaconOption, areEqual);