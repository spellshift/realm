import { XMarkIcon } from "@heroicons/react/24/outline";
import React, { FC, Fragment } from "react";
import { Dialog, Transition } from '@headlessui/react';
import Button from "./button/Button";

const ModalWidth = {
    sm: "lg:max-w-xl",
    md: "lg:max-w-2xl",
    lg: "lg:max-w-4xl"
}
type ModalProps = {
    isOpen: boolean,
    setOpen: (arg: any) => any,
    children: React.ReactElement
    size?: "sm" | "md" | "lg"
}

const Modal: FC<ModalProps> = ({ isOpen, setOpen, size = "md", children }) => {
    const modalSize = ModalWidth[size];

    return (
        <Transition.Root show={isOpen} as={Fragment}>
            <Dialog as="div" className="relative z-10" onClose={setOpen}>
                <div className="fixed inset-0 bg-black/30" aria-hidden="true" />
                <div className="fixed inset-0 overflow-hidden">
                    <div className="absolute inset-0 overflow-hidden">
                        <div className="pointer-events-none fixed inset-y-0 right-0 flex max-w-full pl-4 ">
                            <Transition.Child
                                as={Fragment}
                                enter="transform transition ease-in-out duration-500 sm:duration-700"
                                enterFrom="translate-x-full"
                                enterTo="translate-x-0"
                                leave="transform transition ease-in-out duration-500 sm:duration-700"
                                leaveFrom="translate-x-0"
                                leaveTo="translate-x-full"
                            >
                                <Dialog.Panel className={`pointer-events-auto w-screen max-w-xs md:max-w-md ${modalSize}`}>
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
                                </Dialog.Panel>
                            </Transition.Child>
                        </div>
                    </div>
                </div>
            </Dialog>
        </Transition.Root>
    );
};
export default Modal;
