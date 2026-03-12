import React, { createContext, useCallback, useContext, useRef, useState } from 'react';
import { CreateQuestModal } from '../../components/create-quest-modal';
import {
    CreateQuestModalContextType,
    OpenCreateQuestModalOptions,
} from './types';

const CreateQuestModalContext = createContext<CreateQuestModalContextType | undefined>(undefined);

/* NOTE
*
*  Planned behavior
*  Create Quest Buttons next to header will reroute to quest/task/id
*  Create Quest Buttons located in other locations will not reroute and stay in view
*
*/

export function CreateQuestModalProvider({ children }: { children: React.ReactNode }) {
    const [isOpen, setIsOpen] = useState(false);
    const optionsRef = useRef<OpenCreateQuestModalOptions | undefined>(undefined);

    const openModal = useCallback((options?: OpenCreateQuestModalOptions) => {
        optionsRef.current = options;
        setIsOpen(true);
    }, []);

    const closeModal = useCallback(() => {
        setIsOpen(false);
        optionsRef.current = undefined;
    }, []);

    const handleComplete = useCallback((questId: string) => {
        if (optionsRef.current?.onComplete) {
            optionsRef.current.onComplete(questId);
        }
        closeModal();
    }, [closeModal]);

    const handleSetOpen = useCallback((open: boolean) => {
        if (!open) {
            closeModal();
        } else {
            setIsOpen(true);
        }
    }, [closeModal]);

    return (
        <CreateQuestModalContext.Provider value={{ isOpen, openModal, closeModal }}>
            {children}
            {isOpen && (
                <CreateQuestModal
                    isOpen={isOpen}
                    setOpen={handleSetOpen}
                    initialFormData={optionsRef.current?.initialFormData}
                    onComplete={handleComplete}
                />
            )}
        </CreateQuestModalContext.Provider>
    );
}

export function useCreateQuestModal(): CreateQuestModalContextType {
    const context = useContext(CreateQuestModalContext);
    if (context === undefined) {
        throw new Error('useCreateQuestModal must be used within a CreateQuestModalProvider');
    }
    return context;
}
