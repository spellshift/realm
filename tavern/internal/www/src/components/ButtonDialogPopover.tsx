import React, { ReactElement, useEffect, useRef, useState } from 'react';
import { Dialog } from '@headlessui/react';
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

    const buttonRef = useRef<HTMLButtonElement | null>(null);
    const panelRef = useRef<HTMLDivElement | null>(null);

    const openDialog = (): void => {
        const dialogWidth = 384;
        const spacing = 8;

        const button = buttonRef.current
        if (button) {
            const rect = button.getBoundingClientRect()
            const viewportWidth = window.innerWidth

            const calculatedLeft = Math.min(
                rect.left + window.scrollX,
                viewportWidth - dialogWidth - 16 // 16px right margin
            )

            setPosition({
                top: rect.bottom + window.scrollY + spacing,
                left: calculatedLeft,
            })
        }
        setIsOpen(true)
    }

    const closeDialog = (): void => setIsOpen(false)

    useEffect(() => {
        if (!isOpen) return

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
    }, [isOpen])

    return (
        <div className='flex justify-end'>
            <Button ref={buttonRef} leftIcon={leftIcon} buttonVariant='ghost' buttonStyle={{ color: "gray", size: "md" }} onClick={openDialog}>{label}</Button>
            <Dialog open={isOpen} onClose={closeDialog}>
                <div>
                    <div className="fixed inset-0 bg-black/30 z-40" aria-hidden="true" />
                    <div
                        ref={panelRef}
                        className="absolute z-50 bg-white border rounded-lg shadow-lg w-96 p-4 flex flex-col gap-4"
                        style={{
                            top: position.top,
                            left: position.left,
                        }}
                    >
                        {children}
                    </div>
                </div>
            </Dialog>
        </div>
    );
}
