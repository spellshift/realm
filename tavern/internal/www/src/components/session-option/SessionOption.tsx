import React from "react";
import { Text, Stack, StackItem, Card, CardBody, Checkbox, Flex } from "@chakra-ui/react";

export function areEqual(prevProps: any, nextProps: any) {
    const session = prevProps.sessionsToDisplay[prevProps.index];
    const nextSession = nextProps.sessionsToDisplay[nextProps.index];
    return prevProps.sessionsToDisplay === nextProps.sessionsToDisplay && prevProps.sessionsSelected[session.id] === nextProps.sessionsSelected[nextSession.id];
}

type Props = {
    index: number;
    style: any;
    sessionsToDisplay: Array<any>;
    toggleCheck: (arg: any) => void;
    sessionsSelected: any;
};

export const SessionOption = (props: Props) => {
    const {index, style, sessionsToDisplay, toggleCheck, sessionsSelected } = props;
    // Your card component goes here
    const session = sessionsToDisplay[index];
    const group = (session?.tags).find( (obj : any) => {
        return obj?.kind === "group"
    });
    const service = (session?.tags).find( (obj : any) => {
        return obj?.kind === "service"
    });
    const isChecked = sessionsSelected[session.id];

    return (
        <div style={style} key={`session_option_${session.id}`}>
            <Card>
                <CardBody>
                    <Checkbox colorScheme={"purple"} size="lg" isChecked={isChecked} onChange={()=> toggleCheck(session.id)}>
                        <Stack ml={4} w={"xl"}>
                            <StackItem>
                                    <Text fontSize={"md"}>{session.name}</Text> 
                            </StackItem>
                            <StackItem>
                                <Flex direction="row" wrap={"wrap"}>
                                    <Text fontSize={"sm"}>
                                        {group?.name} | {service?.name} {session.principal && `| ${session.principal}`}
                                    </Text>
                                </Flex>
                            </StackItem>
                        </Stack>
                    </Checkbox>
                </CardBody>
            </Card>
        </div>
    );
};

export default React.memo(SessionOption, areEqual);