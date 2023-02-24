import { Box, Button, Heading, ListItem, Stack, StackItem, Textarea } from "@chakra-ui/react";
import React, { useState } from "react";
import { StepIcon } from "../step-icon";

type StepCustomizeParametersParams = {
    step: number;
    currStep: number;
    setCurrStep: any;
}
export const StepCustomizeParameters = (props: StepCustomizeParametersParams) => {
    const {step, currStep, setCurrStep} = props;

    const [value, setValue] = useState('');

    let handleInputChange = (e: any) => {
      let inputValue = e.target.value
      setValue(inputValue)
    };

    const handleNext = () => {
        setCurrStep(step +1);
    };

    const handleBack = () => {
        setCurrStep(step -1);
    }

    return (
        <ListItem>
        <StepIcon step={step} currStep={currStep}/>
        Customize tome parameters
        {currStep === step &&
            <Box px={8} pt={4}>
              <Stack gap={4}>
                <StackItem>
                  <Heading size={"sm"} pb={2}>
                    Tome command
                  </Heading>
                  <Textarea
                      placeholder='cat /etc/passwd'
                      size='xs'
                      value={value}
                      onChange={handleInputChange}
                    />
                </StackItem>
                <StackItem>
                  <Stack direction={"row"} gap={1}>
                  <StackItem>
                    <Button onClick={handleBack}>Back</Button>
                  </StackItem>
                  <StackItem>
                    <Button variant="solid" colorScheme={"purple"} onClick={handleNext}>Next</Button>
                  </StackItem>
                </Stack>
                </StackItem>
              </Stack>
            </Box>
        }
      </ListItem> 
    );
}