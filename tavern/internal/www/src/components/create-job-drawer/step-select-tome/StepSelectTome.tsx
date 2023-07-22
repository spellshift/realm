import { Box, Button, Card, CardBody, Checkbox, Flex, Heading, ListItem, Spacer, Stack, StackItem, Text } from "@chakra-ui/react";
import React, { useEffect } from "react";
import { StepIcon } from "../step-icon";

type StepSelectTomeParams = {
    step: number;
    currStep: number;
    setCurrStep: any;
    formik: any;
}
export const StepSelectTome = (props: StepSelectTomeParams) => {
    const {step, currStep, setCurrStep, formik} = props;

    const handleNext = () => {
        formik.setFieldValue('tomeId', "test");
        setCurrStep(step +1);
    };

    return (
        <ListItem>
            <StepIcon step={step} currStep={currStep} />
            Select a tome
            {step === currStep &&
                <Box px={8} pt={4}>
                    <Card>
                        <CardBody>
                            <Checkbox colorScheme={"purple"} size="lg" isChecked={true} onChange={()=>null}>
                                <Stack ml={4}>
                                    <StackItem>
                                        <Heading size={"sm"}>Shell Execute</Heading>
                                    </StackItem>
                                    <StackItem>
                                        <Text fontSize="xs">Execute a shell script using the default interpreter. /bin/bash for macos & linux, and cmd.exe for windows.</Text>
                                    </StackItem>
                                </Stack>
                            </Checkbox>
                        </CardBody>
                    </Card>
                    <Button my={2} variant="solid" colorScheme={"purple"} onClick={()=> handleNext()}>Next</Button>
                </Box>
            }
        </ListItem>
    );
}