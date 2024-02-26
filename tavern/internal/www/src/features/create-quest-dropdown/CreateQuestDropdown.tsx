import { Menu, Transition } from '@headlessui/react'
import { Fragment } from 'react'
import { ChevronDownIcon } from '@heroicons/react/20/solid'
import { Button } from '@chakra-ui/react'
import { EllipsisHorizontalIcon } from '@heroicons/react/24/outline'
import { Task, Tome } from '../../utils/consts'
import { useCreateQuest } from './useCreateQuest'


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
    tome: Tome,
    tasks: Array<Task>
}) => {
    const {
        handleCreateQuestWithNewTome,
        handleCreateQuestWithSameTome
    } = useCreateQuest();

    return (
        <Menu as="div" >
            <div>
                <Menu.Button className="inline-flex w-full justify-center">
                    {showLabel ?
                        <Button size={"sm"} rightIcon={<ChevronDownIcon
                            className="h-5 w-5"
                            aria-hidden="true"
                        />}>
                            Re-run quest
                        </Button>
                        :
                        <Button size={"sm"} leftIcon={<EllipsisHorizontalIcon
                            className="h-5 w-5"
                            aria-hidden="true"
                        />} />
                    }
                </Menu.Button>
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
                            {({ active }) => (
                                <button
                                    onClick={() => handleCreateQuestWithSameTome(name, originalParms, tome, tasks)}
                                    className={`${active ? 'bg-purple-700 text-white' : 'text-gray-900'
                                        } group flex w-full items-center rounded-md px-2 py-2 text-sm`}
                                >
                                    Re-run with online beacons
                                </button>
                            )}
                        </Menu.Item>
                        <Menu.Item>
                            {({ active }) => (
                                <button
                                    onClick={() => handleCreateQuestWithNewTome(name, tasks)}
                                    className={`${active ? 'bg-purple-700 text-white' : 'text-gray-900'
                                        } group flex w-full items-center rounded-md px-2 py-2 text-sm`}
                                >
                                    Re-run with new tome
                                </button>
                            )}
                        </Menu.Item>
                    </div>
                </Menu.Items>
            </Transition>
        </Menu>
    )
};
