export interface ShellState {
    inputBuffer: string;
    cursorPos: number;
    history: string[];
    historyIndex: number;
    prompt: string;
    isSearching: boolean;
    searchQuery: string;
    currentBlock: string;
}

export interface CompletionState {
    list: string[];
    start: number;
    show: boolean;
    index: number;
}
