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
