import React from "react";
import { Card, CardBody, Checkbox } from "@chakra-ui/react";
import BeaconTile from "../BeaconTile";

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
    const { index, style, beaconsToDisplay, toggleCheck, beaconsSelected } = props;
    // Your card component goes here
    const beacon = beaconsToDisplay[index];
    const isChecked = beaconsSelected[beacon.id];

    return (
        <div style={style} key={`beacon_option_${beacon.id}`}>
            <Card>
                <CardBody>
                    <Checkbox colorScheme={"purple"} size="lg" isChecked={isChecked} onChange={() => toggleCheck(beacon.id)}>
                        <div className="ml-2"><BeaconTile beacon={beacon} /></div>
                    </Checkbox>
                </CardBody>
            </Card>
        </div>
    );
};

export default React.memo(BeaconOption, areEqual);
