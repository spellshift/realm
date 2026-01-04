import React, { ReactElement, useEffect, useRef, useState, Fragment } from 'react';
import { Dialog, Transition } from '@headlessui/react';
import { XMarkIcon } from '@heroicons/react/24/outline';
import Button from './tavern-base-ui/button/Button';

type Position = {
    top: number
    left: number
}

export const ButtonDialogPopover = ({ children, label, leftIcon }: {
    children: ReactElement,
    label: string,
    leftIcon: ReactElement
}) => {
    const [isOpen, setIsOpen] = useState(false)
    const [position, setPosition] = useState<Position>({ top: 0, left: 0 })
    const [isMobile, setIsMobile] = useState(false)

    const buttonRef = useRef<HTMLButtonElement | null>(null);
    const panelRef = useRef<HTMLDivElement | null>(null);

    useEffect(() => {
        const checkMobile = () => {
            setIsMobile(window.innerWidth < 768) // md breakpoint
        }

        checkMobile()
        window.addEventListener('resize', checkMobile)
        return () => window.removeEventListener('resize', checkMobile)
    }, [])

    const openDialog = (): void => {
        if (!isMobile) {
            const dialogWidth = 384;
            const spacing = 8;

            const button = buttonRef.current
            if (button) {
                const rect = button.getBoundingClientRect()
                const viewportWidth = window.innerWidth

                const calculatedLeft = Math.min(
                    rect.left,
                    viewportWidth - dialogWidth - 16
                )

                setPosition({
                    top: rect.bottom + spacing,
                    left: calculatedLeft,
                })
            }
        }
        setIsOpen(true)
    }

    const closeDialog = (): void => setIsOpen(false)

    useEffect(() => {
        if (!isOpen || isMobile) return

        const handleClickOutside = (event: MouseEvent): void => {
            const target = event.target as Node
            if (
                panelRef.current &&
                !panelRef.current.contains(target) &&
                buttonRef.current &&
                !buttonRef.current.contains(target)
            ) {
                closeDialog()
            }
        }

        document.addEventListener('mousedown', handleClickOutside)
        return () => document.removeEventListener('mousedown', handleClickOutside)
    }, [isOpen, isMobile])

    return (
        <div className='flex justify-end'>
            <Button
                ref={buttonRef}
                leftIcon={leftIcon}
                buttonVariant='ghost'
                buttonStyle={{ color: "gray", size: "md" }}
                onClick={openDialog}
            >
                {label}
            </Button>

            <Transition.Root show={isOpen} as={Fragment}>
                <Dialog as="div" className="relative z-50" onClose={closeDialog}>
                    {/* Backdrop */}
                    <Transition.Child
                        as={Fragment}
                        enter="ease-out duration-300"
                        enterFrom="opacity-0"
                        enterTo="opacity-100"
                        leave="ease-in duration-200"
                        leaveFrom="opacity-100"
                        leaveTo="opacity-0"
                    >
                        <div className="fixed inset-0 bg-black/30 z-40" aria-hidden="true" />
                    </Transition.Child>

                    {isMobile ? (
                        /* Mobile: Full-screen modal */
                        <div className="fixed inset-0 z-50 overflow-y-auto">
                            <Transition.Child
                                as={Fragment}
                                enter="ease-out duration-300"
                                enterFrom="opacity-0 scale-95"
                                enterTo="opacity-100 scale-100"
                                leave="ease-in duration-200"
                                leaveFrom="opacity-100 scale-100"
                                leaveTo="opacity-0 scale-95"
                            >
                                <Dialog.Panel className="min-h-full bg-white">
                                    {/* Header with close button */}
                                    <div className="sticky top-0 z-10 bg-white border-b border-gray-200 px-4 py-3 flex items-center justify-between">
                                        <Dialog.Title className="text-lg font-medium text-gray-900">
                                            {label}
                                        </Dialog.Title>
                                        <button
                                            onClick={closeDialog}
                                            className="text-gray-400 hover:text-gray-500"
                                        >
                                            <XMarkIcon className="h-6 w-6" />
                                        </button>
                                    </div>

                                    {/* Content */}
                                    <div className="px-4 py-4">
                                        {children}
                                    </div>
                                </Dialog.Panel>
                            </Transition.Child>
                        </div>
                    ) : (
                        /* Desktop: Positioned popover */
                        <div className="fixed inset-0 z-50 overflow-y-auto">
                            <Transition.Child
                                as={Fragment}
                                enter="ease-out duration-200"
                                enterFrom="opacity-0 scale-95"
                                enterTo="opacity-100 scale-100"
                                leave="ease-in duration-150"
                                leaveFrom="opacity-100 scale-100"
                                leaveTo="opacity-0 scale-95"
                            >
                                <div
                                    ref={panelRef}
                                    className="fixed bg-white border rounded-lg shadow-lg w-96 p-4 flex flex-col gap-4"
                                    style={{
                                        top: position.top,
                                        left: position.left,
                                    }}
                                >
                                    {children}
                                </div>
                            </Transition.Child>
                        </div>
                    )}
                </Dialog>
            </Transition.Root>
        </div>
    );
}
