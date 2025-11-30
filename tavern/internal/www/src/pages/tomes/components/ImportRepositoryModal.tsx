import { FC, useState } from "react";
import { FormSteps } from "../../../components/form-steps";
import Modal from "../../../components/tavern-base-ui/Modal";
import { RepositoryNode, TomeEdge } from "../../../utils/interfacesQuery";
import StepAddDeploymentKey from "./StepAddDeploymentKey";
import StepCreateRepository from "./StepCreateRepository";

type ImportRepositoryModalProps = {
    isOpen: boolean,
    setOpen: (arg: any) => any,
}
const defaultNewRepository: RepositoryNode = {
    id: "",
    lastModifiedAt: "",
    url: "",
    publicKey: "",
    tomes: {
        edges: [] as TomeEdge[]
    },
    owner: null
};

const ImportRepositoryModal: FC<ImportRepositoryModalProps> = ({ isOpen, setOpen }) => {
    const [newRepository, setNewRepository] = useState<RepositoryNode>(defaultNewRepository);

    const [currStep, setCurrStep] = useState(0);

    const steps = [
        { name: 'Link repository', description: 'Step 1', href: '#', step: 0 },
        { name: 'Add public key', description: 'Step 2', href: '#', step: 1 },
    ];

    function getStepView(step: number) {
        switch (step) {
            case 0:
                return <StepCreateRepository setCurrStep={setCurrStep} setNewRepository={setNewRepository} />
            case 1:
                return <StepAddDeploymentKey setCurrStep={setCurrStep} newRepository={newRepository} setOpen={setOpen} />
            default:
                return <div>An error has occured</div>;
        }
    }

    return (
        <Modal setOpen={setOpen} isOpen={isOpen} size="lg">
            <div className="flex flex-col gap-12">
                <div>
                    <h3 className="text-xl font-semibold leading-6 text-gray-900">Import tome repository</h3>
                </div>
                <div className="grid grid-cols-12">
                    <div className="hidden md:flex col-span-3">
                        <FormSteps currStep={currStep} steps={steps} />
                    </div>
                    <div className="col-span-12 md:col-span-9">
                        {getStepView(currStep)}
                    </div>
                </div>
            </div>
        </Modal>
    );
};
export default ImportRepositoryModal;
