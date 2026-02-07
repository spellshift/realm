import { differenceInSeconds } from "date-fns";

export function formatRelativeTime(date: Date | string | number): string {
    const now = new Date();
    const targetDate = new Date(date);
    const diffInSeconds = Math.abs(differenceInSeconds(now, targetDate));

    if (diffInSeconds < 60) {
        return `${diffInSeconds}s ago`;
    }

    const minutes = Math.floor(diffInSeconds / 60);
    if (diffInSeconds < 3600) {
        return `${minutes}m ago`;
    }

    const hours = Math.floor(diffInSeconds / 3600);
    if (diffInSeconds < 86400) {
        const remainingMinutes = minutes % 60;
        if (remainingMinutes === 0) {
            return `${hours}h ago`;
        }
        return `${hours}h ${remainingMinutes}m ago`;
    }

    return ">1d ago";
}
