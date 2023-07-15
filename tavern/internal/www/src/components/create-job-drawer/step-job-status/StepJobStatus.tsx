import { Box, Button, Heading, ListItem, Stack, StackItem, Textarea } from "@chakra-ui/react";
import React, { useState } from "react";
import { StepIcon } from "../step-icon";

type StepJobStatusParams = {
    step: number;
    currStep: number;
    setCurrStep: any;
    onClose: any;
}
export const StepJobStatus = (props: StepJobStatusParams) => {
    const {step, currStep, setCurrStep, onClose} = props;
    const overrideStepIcon = step === currStep ? currStep +1 : currStep;

    const handleBack = () => {
        setCurrStep(step -1);
    }

    return (
        <ListItem>
            <StepIcon step={step} currStep={overrideStepIcon}/>
            Job Finalized
            {currStep === step &&
                <Box px={8} pt={4}>
                <Stack gap={4}>
                    <StackItem>
                        Each task created gets listed here
                    </StackItem>
                    <StackItem>
                    <Stack direction={"row"} gap={1}>
                    <StackItem>
                        <Button onClick={handleBack}>Back</Button>
                    </StackItem>
                    <StackItem>
                        <Button variant="solid" colorScheme={"purple"} onClick={onClose}>Finish</Button>
                    </StackItem>
                    </Stack>
                    </StackItem>
                </Stack>
                </Box>
            }
      </ListItem> 
    );
}