import { Button, Card, CardBody, Checkbox, Drawer, DrawerBody, DrawerCloseButton, DrawerContent, DrawerFooter, DrawerHeader, DrawerOverlay, Flex, Heading, Input, Spacer, Stack, useDisclosure, Text, List, ListItem, ListIcon, Box, CardFooter, StackItem, Alert, AlertIcon, Textarea } from "@chakra-ui/react";
import { PhoneIcon, AddIcon, WarningIcon, CheckCircleIcon } from '@chakra-ui/icons'
import React, {useState} from "react";
import { AiOutlineCheckCircle } from "react-icons/ai";
import { StepSelectTome } from "./step-select-tome/StepSelectTome";
import { StepCustomizeParameters } from "./step-customize-parameters/StepCustomizeParameters";
import { StepSelectTargets } from "./step-select-targets/StepSelectTargets";
import { StepJobStatus } from "./step-job-status";

export const CreateJobDrawer = () => {
    const { isOpen, onOpen, onClose } = useDisclosure();
    const [currStep, setCurrStep] = useState<number>(0);
    

    return (
      <>
        <Button onClick={onOpen}>Open</Button>
        <Drawer isOpen={isOpen} onClose={onClose} size="lg">
          <DrawerOverlay />
          <DrawerContent>
            <DrawerCloseButton />
            <DrawerHeader>
                Start a new job
            </DrawerHeader>
  
            <DrawerBody>
              <form
                id='my-form'
                onSubmit={(e) => {
                  e.preventDefault()
                  console.log('submitted')
                }}
              >
                <List spacing={3}>
                  <StepSelectTome step={0} currStep={currStep} setCurrStep={setCurrStep}/>
                  <StepCustomizeParameters step={1} currStep={currStep} setCurrStep={setCurrStep}/>                 
                  <StepSelectTargets step={2} currStep={currStep} setCurrStep={setCurrStep}/>
                  <StepJobStatus step={3} currStep={currStep} setCurrStep={setCurrStep} onClose={onClose}/>
                </List>
                {/* <Input name='nickname' placeholder='Type here...' /> */}
              </form>
            </DrawerBody>
  
            {/* <DrawerFooter>
              <Button type='submit' form='my-form'>
                Save
              </Button>
            </DrawerFooter> */}
          </DrawerContent>
        </Drawer>
      </>
    );
};