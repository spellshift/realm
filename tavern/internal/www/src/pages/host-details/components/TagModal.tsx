import { FC } from "react";
import Modal from "../../../components/tavern-base-ui/Modal";
import PageHeader from "../../../components/tavern-base-ui/PageHeader";

const TagModal: FC<any> = ({ isOpen, setOpen }) => {

    return (
        <Modal setOpen={setOpen} isOpen={isOpen} size="sm">
            <div className="flex flex-col gap-4">
                <PageHeader title="Update tags" />
                <div>
                    Testing
                </div>
            </div>
        </Modal>
    );
};
export default TagModal;
