import React from 'react';
import { useQuery, useMutation } from '@apollo/client';
import { Link } from 'react-router-dom';
import { BellIcon, BugAntIcon } from '@heroicons/react/24/outline';
import {
    Popover,
    PopoverTrigger,
    PopoverContent,
    PopoverHeader,
    PopoverBody,
    PopoverArrow,
    Tabs,
    TabList,
    TabPanels,
    Tab,
    TabPanel,
    LinkBox,
    LinkOverlay,
    Box,
    Text,
    VStack,
    HStack,
} from '@chakra-ui/react';
import { GET_NOTIFICATIONS, MARK_NOTIFICATIONS_AS_READ } from '../../lib/notifications';
import { NotificationPriority, EventKind } from '../../utils/enums';
import { NotificationNode } from '../../utils/interfacesQuery';
import { FileTerminal } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';

const NotificationBell = () => {
    const { data, refetch } = useQuery(GET_NOTIFICATIONS, {
        variables: { first: 100 },
        pollInterval: 5000, // Poll every 5 seconds
    });
    const [markAsRead] = useMutation(MARK_NOTIFICATIONS_AS_READ);

    const notifications: NotificationNode[] = data?.me?.notifications?.edges?.map((edge: any) => edge.node) || [];
    const unreadCount = notifications.filter(n => !n.read).length;

    const urgentNotifications = notifications.filter(n => n.priority === NotificationPriority.Urgent && !n.archived);
    const unreadNotifications = notifications.filter(n => !n.read && !n.archived);
    const readNotifications = notifications.filter(n => n.read && !n.archived);
    const archivedNotifications = notifications.filter(n => n.archived);

    const handleClose = () => {
        if (unreadNotifications.length > 0) {
            markAsRead({
                variables: { notificationIDs: unreadNotifications.map(n => n.id) },
                onCompleted: () => refetch(),
            });
        }
    };

    const getEventIcon = (kind: EventKind) => {
        switch (kind) {
            case EventKind.QUEST_COMPLETED:
                return <FileTerminal size={16} />;
            case EventKind.SHELL_CREATED:
                return <FileTerminal size={16} />;
            case EventKind.BEACON_LOST:
            case EventKind.HOST_ACCESS_NEW:
            case EventKind.HOST_ACCESS_RECOVERED:
            case EventKind.HOST_ACCESS_LOST:
                return <BugAntIcon className="h-4 w-4" />;
            default:
                return <BellIcon className="h-4 w-4" />;
        }
    };

    const getEventDescription = (notification: NotificationNode) => {
        const event = notification.event;
        switch (event.kind) {
            case EventKind.HOST_ACCESS_NEW:
                return `New host access: ${event.host?.name || event.host?.id}`;
            case EventKind.HOST_ACCESS_RECOVERED:
                return `Host access recovered: ${event.host?.name || event.host?.id}`;
            case EventKind.HOST_ACCESS_LOST:
                return `Host access lost: ${event.host?.name || event.host?.id}`;
            case EventKind.BEACON_LOST:
                return `Beacon lost: ${event.beacon?.name || event.beacon?.id}`;
            case EventKind.QUEST_COMPLETED:
                return `Quest completed: ${event.quest?.name || event.quest?.id}`;
            case EventKind.SHELL_CREATED:
                return `Shell created${event.user ? ` by ${event.user.name}` : ''}: ${event.shell?.id || 'unknown'}`;
            default:
                return 'Notification received';
        }
    };

    const getNotificationLink = (notification: NotificationNode) => {
        const event = notification.event;
        switch (event.kind) {
            case EventKind.HOST_ACCESS_NEW:
            case EventKind.HOST_ACCESS_RECOVERED:
            case EventKind.HOST_ACCESS_LOST:
                return event.host ? `/hosts/${event.host.id}` : null;
            case EventKind.BEACON_LOST:
                return event.host?.id ? `/hosts/${event.host.id}` : (event.beacon?.host?.id ? `/hosts/${event.beacon.host.id}` : null);
            case EventKind.QUEST_COMPLETED:
                return event.quest ? `/tasks/${event.quest.id}` : null;
            case EventKind.SHELL_CREATED:
                return null;
            default:
                return null;
        }
    };

    const NotificationList = ({ list }: { list: NotificationNode[] }) => {
        if (list.length === 0) {
            return (
                <Box py={10} textAlign="center">
                    <Text color="gray.500">No notifications</Text>
                </Box>
            );
        }

        return (
            <VStack
                align="stretch"
                spacing={0}
                maxH="400px"
                overflowY="auto"
                position="relative"
                sx={{
                    '&::before': {
                        content: '""',
                        position: 'absolute',
                        left: '27px',
                        top: '0px',
                        bottom: '0px',
                        width: '2px',
                        bg: 'gray.700',
                        zIndex: 0
                    }
                }}
            >
                {list.map((n) => {
                    const link = getNotificationLink(n);
                    return (
                        <LinkBox
                            key={n.id}
                            as="article"
                            py={3}
                            px={4}
                            position="relative"
                            zIndex={1}
                            display="block"
                            _hover={link ? { bg: "gray.800" } : {}}
                        >
                            <HStack align="start" spacing={4}>
                                <Box
                                    borderRadius="full"
                                    bg={n.read ? "gray.700" : "purple.900"}
                                    p={2}
                                    color={n.read ? "gray.400" : "purple.200"}
                                    border="2px solid"
                                    borderColor="gray.900"
                                    zIndex={2}
                                >
                                    {getEventIcon(n.event.kind)}
                                </Box>
                                <VStack align="start" spacing={0} flex={1}>
                                    {link ? (
                                        <LinkOverlay as={Link} to={link}>
                                            <Text fontSize="sm" fontWeight={n.read ? "normal" : "semibold"} color="white">
                                                {getEventDescription(n)}
                                            </Text>
                                        </LinkOverlay>
                                    ) : (
                                        <Text fontSize="sm" fontWeight={n.read ? "normal" : "semibold"} color="white">
                                            {getEventDescription(n)}
                                        </Text>
                                    )}
                                    <Text fontSize="xs" color="gray.400">
                                        {formatDistanceToNow(new Date(n.createdAt), { addSuffix: true })}
                                    </Text>
                                </VStack>
                            </HStack>
                        </LinkBox>
                    );
                })}
            </VStack>
        );
    };

    const activeTabs: string[] = [];
    if (urgentNotifications.length > 0) activeTabs.push('Urgent');
    if (unreadNotifications.length > 0) activeTabs.push('Unread');
    if (readNotifications.length > 0) activeTabs.push('Read');
    if (archivedNotifications.length > 0) activeTabs.push('Archived');

    return (
        <Popover
            placement="right-start"
            onClose={handleClose}
        >
            <PopoverTrigger>
                <Box position="relative" cursor="pointer" p={2} borderRadius="md" _hover={{ bg: "gray.800" }}>
                    <BellIcon className="h-6 w-6 text-gray-400 hover:text-white" />
                    {unreadCount > 0 && (
                        <Box
                            position="absolute"
                            top="0"
                            right="0"
                            bg="purple.700"
                            color="white"
                            fontSize="10px"
                            fontWeight="bold"
                            borderRadius="full"
                            w="18px"
                            h="18px"
                            display="flex"
                            alignItems="center"
                            justifyContent="center"
                            border="2px solid #111827"
                        >
                            {unreadCount > 99 ? '99+' : unreadCount}
                        </Box>
                    )}
                </Box>
            </PopoverTrigger>
            <PopoverContent bg="gray.900" borderColor="gray.700" color="white" w="350px" boxShadow="xl" _focus={{ outline: 'none' }}>
                <PopoverArrow bg="gray.900" />
                <Box p={4} borderBottomWidth="1px" borderColor="gray.700">
                    <Text fontWeight="bold">Notifications</Text>
                </Box>
                <Box p={0}>
                    {activeTabs.length > 0 ? (
                        <Tabs colorScheme="purple" isFitted>
                            <TabList borderColor="gray.700">
                                {activeTabs.map(tab => (
                                    <Tab key={tab} fontSize="xs" py={3} _focus={{ outline: 'none' }}>{tab}</Tab>
                                ))}
                            </TabList>
                            <TabPanels>
                                {activeTabs.map(tab => (
                                    <TabPanel key={tab} p={0}>
                                        <NotificationList list={
                                            tab === 'Urgent' ? urgentNotifications :
                                                tab === 'Unread' ? unreadNotifications :
                                                    tab === 'Read' ? readNotifications :
                                                        archivedNotifications
                                        } />
                                    </TabPanel>
                                ))}
                            </TabPanels>
                        </Tabs>
                    ) : (
                        <Box py={10} textAlign="center">
                            <Text color="gray.500">No notifications at all</Text>
                        </Box>
                    )}
                </Box>
            </PopoverContent>
        </Popover>
    );
};

export default NotificationBell;
