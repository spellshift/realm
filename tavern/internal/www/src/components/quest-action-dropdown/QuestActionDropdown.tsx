import { Menu, Transition } from "@headlessui/react";
import { ChevronDownIcon } from "@heroicons/react/20/solid";
import { Fragment } from "react";
import Button from "../tavern-base-ui/button/Button";

interface QuestAction {
    label: string;
    onClick: () => void;
}

interface QuestActionDropdownProps {
    label: string;
    actions: QuestAction[];
    loading?: boolean;
}

export const QuestActionDropdown = ({ label, actions, loading = false }: QuestActionDropdownProps) => (
    <Menu as="div" className="relative">
        <Menu.Button
            as={Button}
            buttonStyle={{ color: "purple", size: "md" }}
            rightIcon={<ChevronDownIcon className="h-5 w-5" aria-hidden="true" />}
            disabled={loading}
        >
            {loading ? "Loading..." : label}
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
            <Menu.Items className="absolute right-0 top-full mt-2 w-40 origin-top-right rounded-md bg-white p-1 shadow-lg ring-1 ring-black/5 focus:outline-none z-10">
                <div>
                    {actions.map((action) => (
                        <Menu.Item key={action.label}>
                            {() => (
                                <Button
                                    buttonVariant="ghost"
                                    buttonStyle={{ color: "gray", size: "sm" }}
                                    className="w-full shadow-none"
                                    onClick={action.onClick}
                                >
                                    {action.label}
                                </Button>
                            )}
                        </Menu.Item>
                    ))}
                </div>
            </Menu.Items>
        </Transition>
    </Menu>
);
