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
    primaryAction?: () => void;
}

export const QuestActionDropdown = ({ label, actions, loading = false, primaryAction }: QuestActionDropdownProps) => (
    <Menu as="div" className="relative inline-flex">
        {primaryAction ? (
            <>
                <Button
                    buttonStyle={{ color: "purple", size: "md" }}
                    className="rounded-r-none pr-2"
                    onClick={primaryAction}
                    disabled={loading}
                >
                    {loading ? "Loading..." : label}
                </Button>
                <Menu.Button
                    as={Button}
                    buttonStyle={{ color: "purple", size: "md" }}
                    className="rounded-l-none pl-2 pr-3"
                    aria-label={`${label} options`}
                    disabled={loading}
                >
                    <ChevronDownIcon className="h-5 w-5" aria-hidden="true" />
                </Menu.Button>
            </>
        ) : (
            <Menu.Button
                as={Button}
                buttonStyle={{ color: "purple", size: "md" }}
                rightIcon={<ChevronDownIcon className="h-5 w-5" aria-hidden="true" />}
                disabled={loading}
            >
                {loading ? "Loading..." : label}
            </Menu.Button>
        )}
        <Transition
            as={Fragment}
            enter="transition ease-out duration-100"
            enterFrom="transform opacity-0 scale-95"
            enterTo="transform opacity-100 scale-100"
            leave="transition ease-in duration-75"
            leaveFrom="transform opacity-100 scale-100"
            leaveTo="transform opacity-0 scale-95"
        >
            <Menu.Items className="absolute right-0 top-full mt-2 w-72 origin-top-right divide-y divide-gray-100 rounded-md bg-white shadow-lg ring-1 ring-black/5 focus:outline-none z-10">
                <div className="px-1 py-1">
                    {actions.map((action) => (
                        <Menu.Item key={action.label}>
                            {() => (
                                <Button
                                    buttonVariant="ghost"
                                    buttonStyle={{ color: "gray", size: "md" }}
                                    className="w-full"
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
