import { format } from "date-fns";
import { ScreenshotNode } from "./types";

export const imageURL = (id: string) => `/cdn/screenshots/${encodeURIComponent(id)}`;

export const thumbnailURL = (id: string) => `/cdn/screenshots/thumbnail/${encodeURIComponent(id)}`;

export const downloadAllURL = (hostId: string) => `/cdn/screenshots/all/${encodeURIComponent(hostId)}`;

export const capturedAt = (value: string) => format(new Date(value), "yyyy-MM-dd HH:mm");

export const sourceLabel = (screenshot: ScreenshotNode) => {
    if (screenshot.task) return screenshot.task.quest.name;
    if (screenshot.shellTask) return "Shell task";
    return "—";
};
