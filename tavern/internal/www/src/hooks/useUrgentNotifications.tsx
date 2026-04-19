import React, { useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { useToast, Box, Text, CloseButton, HStack, VStack } from '@chakra-ui/react';
import { NotificationNode } from '../utils/interfacesQuery';
import { NotificationPriority } from '../utils/enums';
import { getNotificationLink, getEventDescription } from '../utils/notificationHelpers';
import { requestNotificationPermission, showSystemNotification } from '../utils/systemNotification';

// Module-level Set to deduplicate across multiple component instances
const notifiedIds = new Set<string>();

const PRIORITY_COLORS: Record<string, { bg: string; borderColor: string }> = {
    [NotificationPriority.Urgent]: { bg: 'red.700', borderColor: 'red.500' },
    [NotificationPriority.High]: { bg: 'orange.700', borderColor: 'orange.500' },
    [NotificationPriority.Medium]: { bg: 'yellow.700', borderColor: 'yellow.500' },
    [NotificationPriority.Low]: { bg: 'blue.700', borderColor: 'blue.500' },
};

interface NotificationToastProps {
    title: string;
    description: string;
    link: string | null;
    priority: NotificationPriority;
    onNavigate: (path: string) => void;
    onClose?: () => void;
}

const NotificationToast = ({ title, description, link, priority, onNavigate, onClose }: NotificationToastProps) => {
    const colors = PRIORITY_COLORS[priority] || PRIORITY_COLORS[NotificationPriority.Urgent];
    return (
        <Box
            role="alert"
            onClick={() => {
                if (link) {
                    onNavigate(link);
                }
                if (onClose) onClose();
            }}
            cursor={link ? 'pointer' : 'default'}
            bg={colors.bg}
            borderLeft="4px solid"
            borderColor={colors.borderColor}
            color="white"
            p={4}
            borderRadius="md"
            boxShadow="lg"
            maxW="350px"
        >
            <HStack justify="space-between" align="start">
                <VStack align="start" spacing={1}>
                    <Text fontWeight="bold" fontSize="sm">{title}</Text>
                    <Text fontSize="sm">{description}</Text>
                    {link && <Text fontSize="xs" opacity={0.8}>Click to view</Text>}
                </VStack>
                <CloseButton
                    size="sm"
                    onClick={(e) => {
                        e.stopPropagation();
                        if (onClose) onClose();
                    }}
                />
            </HStack>
        </Box>
    );
};

/**
 * Hook that monitors notifications for urgent priority items and displays
 * toasts when new urgent notifications arrive.
 *
 * Uses a module-level Set to prevent duplicate notifications across multiple
 * component instances (e.g. sidebar variants).
 */
const useUrgentNotifications = (notifications: NotificationNode[]) => {
    const navigate = useNavigate();
    const toast = useToast();
    const initializedRef = useRef(false);

    // Request browser notification permission once on mount.
    useEffect(() => {
        requestNotificationPermission();
    }, []);

    useEffect(() => {
        const urgentNotifications = notifications.filter(
            (n) => n.priority === NotificationPriority.Urgent && !n.read && !n.archived
        );

        // On first load, seed the known IDs without firing alerts
        if (!initializedRef.current) {
            urgentNotifications.forEach((n) => notifiedIds.add(n.id));
            initializedRef.current = true;
            return;
        }

        const newUrgent = urgentNotifications.filter((n) => !notifiedIds.has(n.id));
        if (newUrgent.length === 0) {
            return;
        }

        newUrgent.forEach((notification) => {
            notifiedIds.add(notification.id);
            const description = getEventDescription(notification);
            const link = getNotificationLink(notification);

            // Fire an OS-level system notification (e.g. macOS Notification Center).
            // The tag ensures only one notification is shown across all open tabs.
            showSystemNotification({
                title: '⚠ Urgent Notification',
                body: description,
                tag: `urgent-${notification.id}`,
                url: link || undefined,
                onClick: () => {
                    if (link) {
                        navigate(link);
                    }
                },
            });

            toast({
                position: 'top-right',
                duration: 10000,
                isClosable: true,
                render: ({ onClose }) => (
                    <NotificationToast
                        title="⚠ Urgent Notification"
                        description={description}
                        link={link}
                        priority={notification.priority}
                        onNavigate={navigate}
                        onClose={onClose}
                    />
                ),
            });
        });
    }, [notifications, toast, navigate]);
};

export default useUrgentNotifications;
