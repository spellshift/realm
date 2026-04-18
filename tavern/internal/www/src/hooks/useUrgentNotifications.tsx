import React, { useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { useToast, Box, Text, CloseButton, HStack, VStack } from '@chakra-ui/react';
import { NotificationNode } from '../utils/interfacesQuery';
import { NotificationPriority } from '../utils/enums';
import { getNotificationLink, getEventDescription } from '../utils/notificationHelpers';

// Module-level Set to deduplicate across multiple component instances
const notifiedIds = new Set<string>();

/**
 * Request browser notification permission if the API is available.
 */
const requestSystemNotificationPermission = async (): Promise<void> => {
    if (!('Notification' in window)) {
        return;
    }
    if (Notification.permission !== 'granted' && Notification.permission !== 'denied') {
        await Notification.requestPermission();
    }
};

/**
 * Show a browser system notification for an urgent notification.
 * Clicking the system notification focuses the window and navigates to the link.
 */
const showSystemNotification = (description: string, link: string | null, navigate: (path: string) => void) => {
    if (!('Notification' in window) || Notification.permission !== 'granted') {
        return;
    }
    const systemNotification = new Notification('Realm - Urgent', {
        body: description,
        icon: '/favicon.ico',
        tag: description,
    });
    systemNotification.onclick = () => {
        window.focus();
        if (link) {
            navigate(link);
        }
        systemNotification.close();
    };
};

interface UrgentNotificationToastProps {
    description: string;
    link: string | null;
    onNavigate: (path: string) => void;
    onClose?: () => void;
}

const UrgentNotificationToast = ({ description, link, onNavigate, onClose }: UrgentNotificationToastProps) => (
    <Box
        role="alert"
        onClick={() => {
            if (link) {
                onNavigate(link);
            }
            if (onClose) onClose();
        }}
        cursor={link ? 'pointer' : 'default'}
        bg="orange.700"
        color="white"
        p={4}
        borderRadius="md"
        boxShadow="lg"
        maxW="350px"
    >
        <HStack justify="space-between" align="start">
            <VStack align="start" spacing={1}>
                <Text fontWeight="bold" fontSize="sm">⚠ Urgent Notification</Text>
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

/**
 * Hook that monitors notifications for urgent priority items and displays
 * toasts and system notifications when new urgent notifications arrive.
 *
 * Uses a module-level Set to prevent duplicate notifications across multiple
 * component instances (e.g. sidebar variants).
 */
const useUrgentNotifications = (notifications: NotificationNode[]) => {
    const navigate = useNavigate();
    const toast = useToast();
    const initializedRef = useRef(false);

    // Request system notification permission on mount
    useEffect(() => {
        requestSystemNotificationPermission();
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

            // Show Chakra UI toast with custom render for click-to-navigate
            toast({
                position: 'top-right',
                duration: 10000,
                isClosable: true,
                render: ({ onClose }) => (
                    <UrgentNotificationToast
                        description={description}
                        link={link}
                        onNavigate={navigate}
                        onClose={onClose}
                    />
                ),
            });

            // Attempt system notification
            showSystemNotification(description, link, navigate);
        });
    }, [notifications, toast, navigate]);
};

export default useUrgentNotifications;
