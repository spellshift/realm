import { Button, Drawer, DrawerBody, DrawerCloseButton, DrawerContent, DrawerHeader, DrawerOverlay, useDisclosure, } from "@chakra-ui/react";
import React, {useState} from "react";

import { DrawerForm } from "./drawer-form";


export const CreateJobDrawer = () => {
    const { isOpen, onOpen, onClose } = useDisclosure();

    return (
      <>
        <Button onClick={onOpen}>Start a job</Button>
        <Drawer isOpen={isOpen} onClose={onClose} size="lg" colorScheme="purple" variant={""}>
          <DrawerOverlay />
          <DrawerContent>
            <DrawerCloseButton />
            <DrawerHeader>
                Start a job
            </DrawerHeader>
            <DrawerBody>
              <DrawerForm onClose={onClose} />
            </DrawerBody>
          </DrawerContent>
        </Drawer>
      </>
    );
};