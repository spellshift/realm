// Mirroring tavern/internal/http/shell/websocket.go

export const WebsocketMessageKind = {
    Input: "INPUT",
    Output: "OUTPUT",
    TaskError: "TASK_ERROR",
    Error: "ERROR",
    ControlFlow: "CONTROL_FLOW",
    OutputFromOtherStream: "OUTPUT_FROM_OTHER_STREAM",
} as const;

export type WebsocketMessageKindType = typeof WebsocketMessageKind[keyof typeof WebsocketMessageKind];

export const WebsocketControlFlowSignal = {
    PortalUpgrade: "PORTAL_UPGRADE",
    PortalDowngrade: "PORTAL_DOWNGRADE",
    TaskQueued: "TASK_QUEUED",
} as const;

export type WebsocketControlFlowSignalType = typeof WebsocketControlFlowSignal[keyof typeof WebsocketControlFlowSignal];

export interface WebsocketTaskInputMessage {
    kind: typeof WebsocketMessageKind.Input;
    input: string;
}

export interface WebsocketTaskOutputMessage {
    kind: typeof WebsocketMessageKind.Output;
    output: string;
    shell_task_id: number;
    owner: number;
}

export interface WebsocketTaskErrorMessage {
    kind: typeof WebsocketMessageKind.TaskError;
    error: string;
    shell_task_id: number;
    owner: number;
}

export interface WebsocketErrorMessage {
    kind: typeof WebsocketMessageKind.Error;
    error: string;
}

export interface WebsocketControlFlowMessage {
    kind: typeof WebsocketMessageKind.ControlFlow;
    signal: WebsocketControlFlowSignalType;
    portal_id?: number;
    message?: string;
}

export interface WebsocketTaskOutputFromOtherStreamMessage {
    kind: typeof WebsocketMessageKind.OutputFromOtherStream;
    output: string;
    shell_task_id: number;
    owner: number;
    stream_id: string;
}

export type WebsocketMessage =
    | WebsocketTaskInputMessage
    | WebsocketTaskOutputMessage
    | WebsocketTaskErrorMessage
    | WebsocketErrorMessage
    | WebsocketControlFlowMessage
    | WebsocketTaskOutputFromOtherStreamMessage;
