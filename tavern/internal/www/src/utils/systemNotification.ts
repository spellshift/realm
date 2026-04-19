/**
 * Utilities for sending system-level browser notifications via a service worker.
 *
 * A single service worker instance (notification-sw.js) is shared across all
 * browser tabs. By routing notifications through it, we get natural cross-tab
 * deduplication — only one OS-level notification is shown per event regardless
 * of how many tabs are open.
 *
 * Falls back to the direct Notification API when service workers are unavailable.
 */

/** Cached promise so the service worker is only registered once. */
let swReadyPromise: Promise<ServiceWorkerRegistration | null> | null = null;

/**
 * Register (or retrieve) the notification service worker.
 * The returned promise resolves once the worker is active and ready to
 * receive messages. Returns null if the browser doesn't support service workers.
 */
const getNotificationSW = (): Promise<ServiceWorkerRegistration | null> => {
    if (swReadyPromise) return swReadyPromise;
    if (!('serviceWorker' in navigator)) {
        swReadyPromise = Promise.resolve(null);
        return swReadyPromise;
    }
    swReadyPromise = navigator.serviceWorker
        .register('/notification-sw.js')
        .then(() => navigator.serviceWorker.ready)
        .catch((err) => {
            console.warn('Notification service worker registration failed:', err);
            return null;
        });
    return swReadyPromise;
};

/**
 * Set up a one-time listener for NOTIFICATION_CLICK messages from the
 * service worker so that clicking a notification navigates the focused tab.
 */
let clickListenerAttached = false;
const ensureClickListener = (): void => {
    if (clickListenerAttached || !('serviceWorker' in navigator)) return;
    clickListenerAttached = true;
    navigator.serviceWorker.addEventListener('message', (event) => {
        if (event.data?.type === 'NOTIFICATION_CLICK' && event.data.url) {
            window.focus();
            window.location.href = event.data.url;
        }
    });
};

/**
 * Request permission to show system notifications.
 * Also kicks off service worker registration in the background so it is
 * ready by the time the first notification needs to be shown.
 * Safe to call multiple times — exits early if already granted or denied.
 */
export const requestNotificationPermission = async (): Promise<NotificationPermission> => {
    if (!('Notification' in window)) {
        return 'denied';
    }
    // Start registering the SW early so it's active when needed.
    getNotificationSW();
    ensureClickListener();

    if (Notification.permission === 'granted' || Notification.permission === 'denied') {
        return Notification.permission;
    }
    return Notification.requestPermission();
};

/**
 * Options for creating a system notification.
 */
export interface SystemNotificationOptions {
    title: string;
    body: string;
    /** Optional callback when the notification is clicked (used in fallback path). */
    onClick?: () => void;
    /** Unique tag to deduplicate notifications across tabs via the service worker. */
    tag?: string;
    /** URL to navigate to when the notification is clicked (used by the service worker). */
    url?: string;
}

/**
 * Show a system-level browser notification via the service worker.
 * The service worker deduplicates by tag so only one notification is shown
 * across all open tabs.
 *
 * Falls back to a direct Notification if the service worker is unavailable.
 */
export const showSystemNotification = async ({
    title,
    body,
    onClick,
    tag,
    url,
}: SystemNotificationOptions): Promise<void> => {
    if (!('Notification' in window) || Notification.permission !== 'granted') {
        return;
    }

    const registration = await getNotificationSW();

    if (registration?.active) {
        // Route through the service worker for cross-tab deduplication.
        registration.active.postMessage({
            type: 'SHOW_NOTIFICATION',
            tag,
            title,
            body,
            icon: '/favicon.ico',
            url,
        });
        return;
    }

    // Fallback: show directly (no cross-tab dedup, but still uses tag for browser-level dedup).
    const notification = new Notification(title, {
        body,
        icon: '/favicon.ico',
        ...(tag ? { tag } : {}),
    });

    if (onClick) {
        notification.onclick = () => {
            window.focus();
            onClick();
            notification.close();
        };
    }
};
