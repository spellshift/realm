import React from "react";
import { Steps, Card, Checkbox } from "@chakra-ui/react";
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
            <Card.Root>
                <Card.Body>
                    <Checkbox.Root colorPalette={"purple"} size="lg" onCheckedChange={() => toggleCheck(beacon.id)} aria-label={`beacon ${beacon.id}`} checked={isChecked}><Checkbox.HiddenInput /><Checkbox.Control><Checkbox.Indicator /></Checkbox.Control><Checkbox.Label>
                        <div className="ml-2"><BeaconTile beacon={beacon} /></div>
                    </Checkbox.Label></Checkbox.Root>
                </Card.Body>
            </Card.Root>
        </div>
    );
};

export default React.memo(BeaconOption, areEqual);
