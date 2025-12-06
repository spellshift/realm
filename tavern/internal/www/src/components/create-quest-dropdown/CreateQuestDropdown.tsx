import { Menu, Transition } from '@headlessui/react'
import { Fragment } from 'react'
import { ChevronDownIcon } from '@heroicons/react/20/solid'
import { EllipsisHorizontalIcon } from '@heroicons/react/24/outline'
import { LimitedTaskNode, useCreateQuest } from './useCreateQuest'
import Button from '../../components/tavern-base-ui/button/Button'
import { TomeNode } from '../../utils/interfacesQuery'


export const CreateQuestDropdown = ({
    showLabel,
    name,
    originalParms,
    tome,
    tasks
}: {
    showLabel?: boolean,
    name: string,
    originalParms: string,
    tome: TomeNode,
    tasks: {
        edges: Array<LimitedTaskNode>
    }
}) => {
    const {
        handleCreateQuestWithNewTome,
        handleCreateQuestWithSameTome
    } = useCreateQuest();

    return (
        <Menu as="div" >
            <div>
                {showLabel ?
                    <Menu.Button
                        as={Button}
                        buttonStyle={{ color: 'purple', size: "md" }}
                        rightIcon={<ChevronDownIcon
                            className="h-5 w-5"
                            aria-hidden="true"
                        />}
                    >
                        Re-run quest
                    </Menu.Button>
                    :
                    <Menu.Button
                        as={Button}
                        leftIcon={<EllipsisHorizontalIcon
                            className="h-5 w-5"
                            aria-hidden="true"
                        />}
                    />
                }
            </div>
            <Transition
                as={Fragment}
                enter="transition ease-out duration-100"
                enterFrom="transform opacity-0 scale-95"
                enterTo="transform opacity-100 scale-100"
                leave="transition ease-in duration-75"
                leaveFrom="transform opacity-100 scale-100"
                leaveTo="transform opacity-0 scale-95"
            >
                <Menu.Items className="absolute right-8 mt-2 w-72 origin-top-right divide-y divide-gray-100 rounded-md bg-white shadow-lg ring-1 ring-black/5 focus:outline-none z-10">
                    <div className="px-1 py-1 ">
                        <Menu.Item>
                            {() => (
                                <Button
                                    buttonVariant="ghost"
                                    buttonStyle={{ color: "gray", size: "md" }}
                                    className="w-full"
                                    onClick={() => handleCreateQuestWithSameTome(name, originalParms, tome, tasks)}
                                >
                                    Re-run with online beacons
                                </Button>
                            )}
                        </Menu.Item>
                        <Menu.Item>
                            {() => (
                                <Button
                                    buttonVariant="ghost"
                                    buttonStyle={{ color: "gray", size: "md" }}
                                    onClick={() => handleCreateQuestWithNewTome(name, tasks)}
                                    className="w-full"
                                >
                                    Re-run with new tome
                                </Button>
                            )}
                        </Menu.Item>
                    </div>
                </Menu.Items>
            </Transition>
        </Menu>
    )
};
