import { FC } from "react";
import Modal from "../../../components/tavern-base-ui/Modal";
import NewBuildProfile from "../NewBuildProfile";
import { BuildProfile, Tome } from "../types";

type NewBuildProfileModalProps = {
    isOpen: boolean;
    setOpen: (open: boolean) => void;
    catalog: Tome[];
    existingProfile?: BuildProfile;
    onSave: (profile: BuildProfile) => Promise<void> | void;
};

export const NewBuildProfileModal: FC<NewBuildProfileModalProps> = ({
    isOpen,
    setOpen,
    catalog,
    existingProfile,
    onSave,
}) => {
    return (
        <Modal isOpen={isOpen} setOpen={setOpen} size="lg">
            <div className="flex flex-col gap-4 pb-6" data-testid="new-profile-modal">
                <NewBuildProfile
                    catalog={catalog}
                    existingProfile={existingProfile}
                    onCancel={() => setOpen(false)}
                    onSave={async (profile) => {
                        await onSave(profile);
                        setOpen(false);
                    }}
                />
            </div>
        </Modal>
    );
};

export default NewBuildProfileModal;
