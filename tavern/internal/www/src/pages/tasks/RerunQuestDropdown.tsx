import { Menu, Transition } from "@headlessui/react";
import { Fragment } from "react";
import { ChevronDownIcon } from "@heroicons/react/20/solid";
import { FileTerminal } from "lucide-react";
import Button from "../../components/tavern-base-ui/button/Button";
import { useCreateQuestModal } from "../../context/CreateQuestModalContext";
import { useRerunQuestFormData } from "./useRerunQuestFormData";
import { QuestNode } from "../../utils/interfacesQuery";

interface RerunQuestDropdownProps {
    quest: QuestNode;
}

export const RerunQuestDropdown = ({ quest }: RerunQuestDropdownProps) => {
    const { openModal } = useCreateQuestModal();
    const { buildFormDataWithSameTome, buildFormDataWithNewTome, loading } =
        useRerunQuestFormData(quest);

    const handleRerunWithOnlineBeacons = async () => {
        const initialFormData = await buildFormDataWithSameTome();
        openModal({
            initialFormData,
            navigateToQuest: true,
        });
    };

    const handleRerunWithNewTome = async () => {
        const initialFormData = await buildFormDataWithNewTome();
        openModal({
            initialFormData,
            navigateToQuest: true,
        });
    };

    return (
        <Menu as="div" className="relative">
            <Menu.Button
                as={Button}
                leftIcon={<FileTerminal className="h-5 w-5" />}
                buttonStyle={{ color: "purple", size: "md" }}
                rightIcon={<ChevronDownIcon className="h-5 w-5" aria-hidden="true" />}
                disabled={loading}
            >
                {loading ? "Loading..." : "Re-run quest"}
            </Menu.Button>
            <Transition
                as={Fragment}
                enter="transition ease-out duration-100"
                enterFrom="transform opacity-0 scale-95"
                enterTo="transform opacity-100 scale-100"
                leave="transition ease-in duration-75"
                leaveFrom="transform opacity-100 scale-100"
                leaveTo="transform opacity-0 scale-95"
            >
                <Menu.Items className="absolute right-0 mt-2 w-72 origin-top-right divide-y divide-gray-100 rounded-md bg-white shadow-lg ring-1 ring-black/5 focus:outline-none z-10">
                    <div className="px-1 py-1">
                        <Menu.Item>
                            {() => (
                                <Button
                                    buttonVariant="ghost"
                                    buttonStyle={{ color: "gray", size: "md" }}
                                    className="w-full"
                                    onClick={handleRerunWithOnlineBeacons}
                                >
                                    Rerun with online beacons
                                </Button>
                            )}
                        </Menu.Item>
                        <Menu.Item>
                            {() => (
                                <Button
                                    buttonVariant="ghost"
                                    buttonStyle={{ color: "gray", size: "md" }}
                                    className="w-full"
                                    onClick={handleRerunWithNewTome}
                                >
                                    Rerun with new tome
                                </Button>
                            )}
                        </Menu.Item>
                    </div>
                </Menu.Items>
            </Transition>
        </Menu>
    );
};
