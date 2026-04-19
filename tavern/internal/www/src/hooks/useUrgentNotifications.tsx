import React, { useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { useMutation } from '@apollo/client';
import { useToast, Box, Text, CloseButton, HStack, VStack } from '@chakra-ui/react';
import { NotificationNode } from '../utils/interfacesQuery';
import { NotificationPriority } from '../utils/enums';
import { getNotificationLink, getEventDescription } from '../utils/notificationHelpers';
import { MARK_NOTIFICATIONS_AS_READ } from '../lib/notifications';
import { initNotificationServiceWorker, showSystemNotification, onServiceWorkerNotificationClick } from '../utils/systemNotification';

const STORAGE_KEY = 'notified-urgent-ids';

/** Load previously-notified IDs from sessionStorage so they survive page refresh. */
function loadNotifiedIds(): Set<string> {
    try {
        const stored = sessionStorage.getItem(STORAGE_KEY);
        return stored ? new Set(JSON.parse(stored)) : new Set();
    } catch {
        return new Set();
    }
}

/** Persist the notified IDs set to sessionStorage. */
function saveNotifiedIds(ids: Set<string>): void {
    try {
        sessionStorage.setItem(STORAGE_KEY, JSON.stringify(Array.from(ids)));
    } catch {
        // Ignore storage errors (e.g. private browsing quota).
    }
}

// Module-level Set to deduplicate across multiple component instances.
// Seeded from sessionStorage so IDs survive page refresh.
const notifiedIds = loadNotifiedIds();

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
    const [markAsRead] = useMutation(MARK_NOTIFICATIONS_AS_READ);

    // Register the notification service worker and request permission once on mount.
    // Also listen for click messages from the SW so we can navigate in this tab.
    useEffect(() => {
        initNotificationServiceWorker();
        const cleanup = onServiceWorkerNotificationClick((url) => {
            if (url) {
                window.focus();
                try {
                    const parsed = new URL(url);
                    navigate(parsed.pathname + parsed.search + parsed.hash);
                } catch {
                    navigate(url);
                }
            }
        });
        return cleanup;
    }, [navigate]);

    useEffect(() => {
        const urgentNotifications = notifications.filter(
            (n) => n.priority === NotificationPriority.Urgent && !n.read && !n.archived
        );

        // On first load, seed the known IDs without firing alerts.
        // Wait until `notifications` is non-empty so we don't mark as
        // initialized before the query has returned data.
        if (!initializedRef.current) {
            if (notifications.length === 0) {
                return;
            }
            urgentNotifications.forEach((n) => notifiedIds.add(n.id));
            saveNotifiedIds(notifiedIds);
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

            // Fire an OS-level system notification via the service worker.
            // The `tag` ensures only one notification per event across all tabs.
            const notificationUrl = link ? `${window.location.origin}${link}` : undefined;
            showSystemNotification({
                title: '⚠ Urgent Notification',
                body: description,
                tag: `urgent-${notification.id}`,
                url: notificationUrl,
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
                        onClose={() => {
                            markAsRead({ variables: { notificationIDs: [notification.id] } });
                            if (onClose) onClose();
                        }}
                    />
                ),
            });
        });

        saveNotifiedIds(notifiedIds);
    }, [notifications, toast, navigate, markAsRead]);
};

export default useUrgentNotifications;
