import { FC, useCallback } from "react";
import { VirtualizedCardItem } from "../../../components/tavern-base-ui/virtualized-card-list";
import { ScreenshotCard } from "./ScreenshotCard";
import { gql } from "@apollo/client";

export const GET_SCREENSHOT_QUERY = gql`
    query GetScreenshot($where: ScreenshotWhereInput) {
        screenshots(where: $where) {
            edges {
                node {
                    id
                    createdAt
                }
            }
        }
    }
`;

interface ScreenshotNode {
    id: string;
    createdAt: string;
}

interface ScreenshotQueryTopLevel {
    screenshots: {
        edges: Array<{
            node: ScreenshotNode;
        }>;
    };
}

interface ScreenshotCardVirtualizedProps {
    itemId: string;
    isVisible: boolean;
}

const ScreenshotCardSkeleton: FC = () => {
    return (
        <div className="rounded-lg shadow border-gray-200 border-2 animate-pulse flex flex-col items-center justify-center bg-gray-100 h-96">
            <div className="w-full h-12 bg-gray-200"></div>
            <div className="flex-1 w-full bg-gray-300"></div>
        </div>
    );
};

export const ScreenshotCardVirtualized: FC<ScreenshotCardVirtualizedProps> = ({
    itemId,
    isVisible,
}) => {
    const getVariables = useCallback((id: string) => ({
        where: { id },
    }), []);

    const extractData = useCallback((response: ScreenshotQueryTopLevel): ScreenshotNode | null => {
        return response?.screenshots?.edges?.[0]?.node ?? null;
    }, []);

    const renderCard = useCallback((screenshot: ScreenshotNode) => {
        return <ScreenshotCard screenshot={screenshot} />;
    }, []);

    const renderSkeleton = useCallback(() => {
        return <ScreenshotCardSkeleton />;
    }, []);

    return (
        <VirtualizedCardItem<ScreenshotNode>
            itemId={itemId}
            query={GET_SCREENSHOT_QUERY}
            getVariables={getVariables}
            renderCard={renderCard}
            renderSkeleton={renderSkeleton}
            extractData={extractData}
            isVisible={isVisible}
        />
    );
};

export default ScreenshotCardVirtualized;
