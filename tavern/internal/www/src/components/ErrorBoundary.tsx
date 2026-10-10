import { Component, ErrorInfo, ReactNode } from "react";

interface Props {
    children: ReactNode;
    fallback?: ReactNode;
}

interface State {
    hasError: boolean;
}

export class ErrorBoundary extends Component<Props, State> {
    state: State = { hasError: false };

    static getDerivedStateFromError(): State {
        return { hasError: true };
    }

    componentDidCatch(error: Error, info: ErrorInfo) {
        console.error(error, info);
    }

    render() {
        if (this.state.hasError) {
            return this.props.fallback ?? (
                <div className="p-4 text-red-500 border border-red-200 rounded-md">
                    Failed to load this card.
                </div>
            );
        }
        return this.props.children;
    }
}
