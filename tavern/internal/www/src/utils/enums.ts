export enum StepStatus {
    Current = "CURRENT",
    Upcoming = "UPCOMING",
    Complete = "COMPLETE",
}
export enum TaskStatus {
    inprogress = "IN-PROGRESS",
    finished = "FINISHED",
    queued = "QUEUED",
}
export enum PageNavItem {
    quests='Quest history',
    documentation='Documentation',
    results='Quest outputs',
    createQuest= 'Create new quest',
}
export enum SupportedPlatforms {
    Windows="Windows",
    Linux='Linux',
    MacOS='MacOS',
    BSD='BSD',
    Unknown='Unknown'
}
export enum TableRowLimit {
    TaskRowLimit=8
}