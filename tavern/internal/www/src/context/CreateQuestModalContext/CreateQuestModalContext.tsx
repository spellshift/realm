import React, { createContext, useCallback, useContext, useState } from 'react';
import { CreateQuestModal, CreateScheduledQuestModal } from '../../components/create-quest-modal';
import {
    CreateQuestModalContextType,
    OpenCreateQuestModalOptions,
} from './types';
import { useNavigate } from 'react-router-dom';

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
    const [options, setOptions] = useState<OpenCreateQuestModalOptions | undefined>(undefined);
    const navigate = useNavigate();

    const openModal = useCallback((nextOptions?: OpenCreateQuestModalOptions) => {
        setOptions(nextOptions);
        setIsOpen(true);
    }, []);

    const closeModal = useCallback(() => {
        setIsOpen(false);
        setOptions(undefined);
    }, []);

    const handleComplete = useCallback((questId: string) => {
        if (options?.onComplete) {
            options.onComplete(questId);
        }
        if (options?.navigateToQuest) {
            navigate(`/tasks/${questId}`);
        }
        closeModal();
    }, [closeModal, navigate, options]);

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
            {isOpen && options?.mode === "scheduled" && (
                <CreateScheduledQuestModal
                    isOpen={isOpen}
                    setOpen={handleSetOpen}
                    scheduledHost={options.scheduledHost}
                />
            )}
            {isOpen && options?.mode !== "scheduled" && (
                <CreateQuestModal
                    isOpen={isOpen}
                    setOpen={handleSetOpen}
                    initialFormData={options?.initialFormData}
                    onComplete={handleComplete}
                    refetchQueries={options?.refetchQueries}
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
