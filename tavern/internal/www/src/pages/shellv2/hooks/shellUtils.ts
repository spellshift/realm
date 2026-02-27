export const moveWordLeft = (buffer: string, cursor: number): number => {
    const beforeCursor = buffer.slice(0, cursor);
    const trimmed = beforeCursor.trimEnd();
    const lastSpace = trimmed.lastIndexOf(" ");
    return lastSpace === -1 ? 0 : lastSpace + 1;
};

export const moveWordRight = (buffer: string, cursor: number): number => {
    const len = buffer.length;
    let i = cursor;

    // 1. If we are at the end, stay there
    if (i >= len) return len;

    // 2. If we are on a space, skip all spaces first
    while (i < len && buffer[i] === ' ') {
        i++;
    }

    // 3. Now skip all non-spaces (the word)
    while (i < len && buffer[i] !== ' ') {
        i++;
    }

    return i;
};
