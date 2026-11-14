import { NetworkStatus, useQuery } from "@apollo/client";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useSorts } from "../../../context/SortContext";
import { PageNavItem } from "../../../utils/enums";
import { OrderByField } from "../../../utils/interfacesQuery";
import { GET_HOST_SCREENSHOTS } from "./queries";
import { PAGE_SIZE } from "./options";
import { GetHostScreenshotsQueryVariables, HostScreenshotsQueryResponse, ScreenshotNode } from "./types";

export const useScreenshots = (hostId: string) => {
    const { sorts } = useSorts();
    const screenshotSort = sorts[PageNavItem.screenshots];
    const [selectedId, setSelectedId] = useState<string | null>(null);
    const [loadingMore, setLoadingMore] = useState(false);
    const loadingMoreRef = useRef(false);

    const queryVariables = useMemo(
        () => getScreenshotsQuery(hostId, screenshotSort),
        [hostId, screenshotSort]
    );

    const { data, error, networkStatus, fetchMore } = useQuery<HostScreenshotsQueryResponse, GetHostScreenshotsQueryVariables>(
        GET_HOST_SCREENSHOTS,
        {
            variables: queryVariables,
            skip: !hostId,
            fetchPolicy: "cache-first",
            notifyOnNetworkStatusChange: true,
        }
    );

    const connection = data?.hosts?.edges?.[0]?.node?.screenshots;
    const screenshots = useMemo(() => connection?.edges?.map(edge => edge.node) || [], [connection]);
    const screenshotIds = useMemo(() => screenshots.map(screenshot => screenshot.id), [screenshots]);
    const selectedIndex = Math.max(0, screenshots.findIndex(screenshot => screenshot.id === selectedId));
    const selected: ScreenshotNode | undefined = screenshots[selectedIndex];
    const hasMore = connection?.pageInfo.hasNextPage ?? false;
    const endCursor = connection?.pageInfo.endCursor;

    // The sort field/direction live in the shared SortContext (set via
    // SortingControls), so react to its changes here instead of wrapping a
    // local setter.
    useEffect(() => {
        setSelectedId(null);
    }, [screenshotSort]);

    // Fetches the next page and resolves to its first screenshot, if any.
    const loadMore = useCallback(async (): Promise<ScreenshotNode | undefined> => {
        if (!hasMore || !endCursor || loadingMoreRef.current) {
            return undefined;
        }
        loadingMoreRef.current = true;
        setLoadingMore(true);
        try {
            const result = await fetchMore({
                variables: { after: endCursor },
            });
            return result.data?.hosts?.edges?.[0]?.node?.screenshots?.edges?.[0]?.node;
        } finally {
            loadingMoreRef.current = false;
            setLoadingMore(false);
        }
    }, [hasMore, endCursor, fetchMore]);

    const selectPrevious = useCallback(() => {
        if (selectedIndex === 0) {
            return;
        }
        setSelectedId(screenshots[selectedIndex - 1].id);
    }, [screenshots, selectedIndex]);

    const selectNext = useCallback(async () => {
        if (selectedIndex < screenshots.length - 1) {
            setSelectedId(screenshots[selectedIndex + 1].id);
            return;
        }
        const firstOfNextPage = await loadMore();
        if (firstOfNextPage) {
            setSelectedId(firstOfNextPage.id);
        }
    }, [screenshots, selectedIndex, loadMore]);

    const isLast = selectedIndex === screenshots.length - 1;

    return {
        screenshots,
        screenshotIds,
        totalCount: connection?.totalCount,
        selected,
        selectedIndex,
        setSelectedId,
        selectPrevious,
        selectNext,
        canSelectPrevious: selectedIndex > 0,
        canSelectNext: !loadingMore && (!isLast || hasMore),
        hasMore,
        loadMore,
        loadingMore,
        initialLoading: networkStatus === NetworkStatus.loading && !data,
        error,
        sort: screenshotSort,
    };
};

const getScreenshotsQuery = (
    hostId: string,
    sort: OrderByField,
): GetHostScreenshotsQueryVariables => ({
    hostId,
    first: PAGE_SIZE,
    orderBy: [sort],
});
