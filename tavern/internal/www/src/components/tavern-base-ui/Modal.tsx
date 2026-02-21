import { XMarkIcon } from "@heroicons/react/24/outline";
import React, { FC } from "react";
import { Drawer, Portal } from "@chakra-ui/react";
import Button from "./button/Button";

type ModalProps = {
    isOpen: boolean,
    setOpen: (arg: any) => any,
    children: React.ReactElement
    size?: "sm" | "md" | "lg"
}

const Modal: FC<ModalProps> = ({ isOpen, setOpen, size = "md", children }) => {
    // Mapping size to Chakra Drawer sizes if needed, or use 'maxW' prop
    // Chakra v3 sizes are xs, sm, md, lg, xl, full.
    // Our custom sizes: sm -> md, md -> lg, lg -> xl roughly
    let chakraSize = "md";
    if (size === "sm") chakraSize = "sm";
    if (size === "lg") chakraSize = "lg";

    return (
        <Drawer.Root open={isOpen} onOpenChange={(e: any) => setOpen(e.open)} placement="end" size={chakraSize as any}>
            <Portal>
                <Drawer.Backdrop />
                <Drawer.Positioner>
                    <Drawer.Content>
                        <div className="flex h-full flex-col overflow-y-scroll bg-white py-6 shadow-xl">
                            <div className="px-4 sm:px-6">
                                <div className="flex w-full justify-end">
                                    <div className="ml-3 flex h-7 items-center">
                                        <Button
                                            type="button"
                                            buttonStyle={{ color: "gray", size: "md" }}
                                            buttonVariant="ghost"
                                            onClick={() => setOpen(false)}
                                            leftIcon={<XMarkIcon className="h-6 w-6" aria-hidden="true" />}
                                        >
                                            <span className="sr-only">Close panel</span>
                                        </Button>
                                    </div>
                                </div>
                            </div>
                            <div className="relative mt-6 flex-1 px-4 sm:px-6 flex flex-col gap-4">
                                {children}
                            </div>
                        </div>
                    </Drawer.Content>
                </Drawer.Positioner>
            </Portal>
        </Drawer.Root>
    );
};
export default Modal;
