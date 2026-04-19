import { NotificationNode } from './interfacesQuery';
import { EventKind } from './enums';

/**
 * Get the navigation link for a notification based on its event type.
 * Returns null if no link is available.
 */
export const getNotificationLink = (notification: NotificationNode): string | null => {
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
        case EventKind.NEW_USER_REQUEST:
            return '/admin';
        default:
            return null;
    }
};

/**
 * Get a human-readable description for a notification event.
 */
export const getEventDescription = (notification: NotificationNode): string => {
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
        case EventKind.NEW_USER_REQUEST:
            return `New user request: ${event.user?.name || 'Unknown user'}`;
        default:
            return 'Notification received';
    }
};
