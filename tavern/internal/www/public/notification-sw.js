/**
 * Service Worker for handling system notifications.
 *
 * Because a service worker is shared across all tabs of the same origin,
 * notifications shown via `registration.showNotification()` are
 * deduplicated by the `tag` option — only one OS-level notification is
 * displayed per unique tag, regardless of how many tabs are open.
 */

/* eslint-disable no-restricted-globals */

// No-op install/activate — this SW only handles notification messages.
self.addEventListener('install', () => self.skipWaiting());
self.addEventListener('activate', (event) => event.waitUntil(self.clients.claim()));

/**
 * Listen for messages from client pages requesting a notification.
 *
 * Expected message shape:
 *   { type: 'SHOW_NOTIFICATION', title: string, body: string, tag: string, url?: string }
 */
self.addEventListener('message', (event) => {
    if (event.data && event.data.type === 'SHOW_NOTIFICATION') {
        const { title, body, tag, url } = event.data;
        self.registration.showNotification(title, {
            body,
            icon: '/favicon.ico',
            tag, // same tag → replaces previous notification, so only one is shown
            data: { url },
        });
    }
});

/**
 * Handle notification click — focus an existing tab (and navigate) or open a new one.
 */
self.addEventListener('notificationclick', (event) => {
    event.notification.close();

    const url = event.notification.data && event.notification.data.url;

    event.waitUntil(
        self.clients.matchAll({ type: 'window', includeUncontrolled: true }).then((clientList) => {
            // Try to focus an existing tab
            for (const client of clientList) {
                if (client.url && 'focus' in client) {
                    client.focus();
                    if (url) {
                        client.postMessage({ type: 'NOTIFICATION_CLICK', url });
                    }
                    return;
                }
            }
            // No existing tab — open a new one
            if (url && self.clients.openWindow) {
                return self.clients.openWindow(url);
            }
        })
    );
});
