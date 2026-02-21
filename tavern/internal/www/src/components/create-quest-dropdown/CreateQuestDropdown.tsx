import { Menu, Portal } from '@chakra-ui/react'
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
        <Menu.Root>
            <Menu.Trigger asChild>
                <div>
                    {showLabel ?
                        <Button
                            buttonStyle={{ color: 'purple', size: "md" }}
                            rightIcon={<ChevronDownIcon
                                className="h-5 w-5"
                                aria-hidden="true"
                            />}
                        >
                            Re-run quest
                        </Button>
                        :
                        <Button
                            leftIcon={<EllipsisHorizontalIcon
                                className="h-5 w-5"
                                aria-hidden="true"
                            />}
                        />
                    }
                </div>
            </Menu.Trigger>
            <Portal>
                <Menu.Positioner>
                    <Menu.Content className="z-10">
                        <Menu.Item value="rerun-online" asChild>
                            <Button
                                buttonVariant="ghost"
                                buttonStyle={{ color: "gray", size: "md" }}
                                className="w-full"
                                onClick={() => handleCreateQuestWithSameTome(name, originalParms, tome, tasks)}
                            >
                                Re-run with online beacons
                            </Button>
                        </Menu.Item>
                        <Menu.Item value="rerun-new" asChild>
                            <Button
                                buttonVariant="ghost"
                                buttonStyle={{ color: "gray", size: "md" }}
                                onClick={() => handleCreateQuestWithNewTome(name, tasks)}
                                className="w-full"
                            >
                                Re-run with new tome
                            </Button>
                        </Menu.Item>
                    </Menu.Content>
                </Menu.Positioner>
            </Portal>
        </Menu.Root>
    )
};
