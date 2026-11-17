import React, { ReactElement, useEffect, useRef, useState, Fragment } from 'react';
import { Dialog, Transition } from '@headlessui/react';
import { XMarkIcon } from '@heroicons/react/24/outline';
import Button from './tavern-base-ui/button/Button';

type Position = {
    top: number
    left: number
}

export interface ButtonDialogPopoverProps {
    children: ReactElement;
    label: string;
    leftIcon: ReactElement;
    theme?: "dark" | "light";
    buttonClassName?: string;
    buttonTestId?: string;
    panelTestId?: string;
    panelWidthPx?: number;
}

export const ButtonDialogPopover = ({
    children,
    label,
    leftIcon,
    theme = "light",
    buttonClassName,
    buttonTestId,
    panelTestId,
    panelWidthPx = 384,
}: ButtonDialogPopoverProps) => {
    const [isOpen, setIsOpen] = useState(false)
    const [position, setPosition] = useState<Position>({ top: 0, left: 0 })
    const [isMobile, setIsMobile] = useState(false)

    const buttonRef = useRef<HTMLButtonElement | null>(null);
    const panelRef = useRef<HTMLDivElement | null>(null);

    useEffect(() => {
        const checkMobile = () => {
            setIsMobile(window.innerWidth < 1024) // lg breakpoint
        }

        checkMobile()
        window.addEventListener('resize', checkMobile)
        return () => window.removeEventListener('resize', checkMobile)
    }, [])

    const openDialog = (): void => {
        if (!isMobile) {
            const spacing = 8;

            const button = buttonRef.current
            if (button) {
                const rect = button.getBoundingClientRect()
                const viewportWidth = window.innerWidth

                const calculatedLeft = Math.max(
                    16,
                    Math.min(rect.left, viewportWidth - panelWidthPx - 16)
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

    const isDark = theme === "dark";
    const mobilePanelClass = isDark ? "min-h-full bg-gray-900 text-gray-200" : "min-h-full bg-white";
    const mobileHeaderClass = isDark
        ? "sticky top-0 z-10 bg-gray-900 border-b border-gray-800 px-4 py-3 flex items-center justify-between"
        : "sticky top-0 z-10 bg-white border-b border-gray-200 px-4 py-3 flex items-center justify-between";
    const mobileTitleClass = isDark
        ? "text-lg font-medium text-gray-100"
        : "text-lg font-medium text-gray-900";
    const desktopPanelClass = isDark
        ? "fixed bg-gray-900 border border-gray-700 text-gray-200 rounded-lg shadow-xl p-4 flex flex-col gap-3"
        : "fixed bg-white border rounded-lg shadow-lg p-4 flex flex-col gap-4";

    return (
        <div className='flex justify-end'>
            <Button
                ref={buttonRef}
                data-testid={buttonTestId}
                leftIcon={leftIcon}
                buttonVariant='ghost'
                buttonStyle={{ color: "gray", size: "md" }}
                className={buttonClassName}
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
                                <Dialog.Panel data-testid={panelTestId} className={mobilePanelClass}>
                                    {/* Header with close button */}
                                    <div className={mobileHeaderClass}>
                                        <Dialog.Title className={mobileTitleClass}>
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
                                    data-testid={panelTestId}
                                    className={desktopPanelClass}
                                    style={{
                                        top: position.top,
                                        left: position.left,
                                        width: panelWidthPx,
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
