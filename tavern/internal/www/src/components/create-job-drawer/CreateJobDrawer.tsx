import { Button, Drawer, DrawerBody, DrawerCloseButton, DrawerContent, DrawerHeader, DrawerOverlay, useDisclosure, } from "@chakra-ui/react";
import React, {useState} from "react";

import { DrawerForm } from "./drawer-form";


export const CreateJobDrawer = () => {
    const { isOpen, onOpen, onClose } = useDisclosure();

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
              <DrawerForm onClose={onClose} />
            </DrawerBody>
          </DrawerContent>
        </Drawer>
      </>
    );
};