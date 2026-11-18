import { fireEvent, render, screen } from '@testing-library/react';
import { print } from 'graphql';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { QuestsTable } from './QuestsTable';
import { GET_QUEST_DETAIL_QUERY } from './queries';
import { QuestDetailNode } from './types';

const mockNavigate = vi.fn();
const mockRows = vi.hoisted(() => ({ current: {} as Record<string, unknown> }));

vi.mock('react-router-dom', () => ({ useNavigate: () => mockNavigate }));
vi.mock('../../context/FilterContext', () => ({ useFilters: () => ({ filters: {} }) }));
vi.mock('../../components/UserImageAndName', () => ({ default: () => <span>Creator</span> }));
vi.mock('../../components/tavern-base-ui/virtualized-table/VirtualizedTable', () => ({
    VirtualizedTable: ({ items, columns, extractData, onItemClick }: any) => (
        <div>
            {items.map((id: string) => {
                const quest = extractData(mockRows.current[id]);
                return <div key={id} data-testid="quest-row" onClick={() => onItemClick(id)}>
                    {columns.map((column: any) => <div key={column.key}>{column.render(quest)}</div>)}
                </div>;
            })}
        </div>
    ),
}));

function questResponse(id: string, occurrence: string | null, taskCount: number) {
    const quest = {
        id,
        name: 'Automated: hourly',
        scheduledOccurrenceAt: occurrence,
        tome: { name: 'Example tome' },
        creator: null,
        tasks: { edges: Array.from({ length: taskCount }, (_, index) => ({ node: { id: `task-${index}` } })) },
        tasksTotal: { totalCount: taskCount },
        tasksFinished: { totalCount: 0 },
        tasksOutput: { totalCount: 0 },
        tasksError: { totalCount: 0 },
        lastUpdatedTask: { edges: [] },
    } as unknown as QuestDetailNode;
    return { quests: { edges: [{ node: quest }] } };
}

describe('QuestsTable scheduled occurrences', () => {
    beforeEach(() => {
        mockNavigate.mockClear();
        mockRows.current = {};
    });

    it('queries the occurrence timestamp and renders an hourly occurrence with two Tasks as one Quest row', () => {
        expect(print(GET_QUEST_DETAIL_QUERY)).toContain('scheduledOccurrenceAt');
        mockRows.current['quest-1'] = questResponse('quest-1', '2026-11-18T15:00:00Z', 2);

        render(<QuestsTable questIds={['quest-1']} />);

        expect(screen.getAllByTestId('quest-row')).toHaveLength(1);
        expect(screen.getByText(/^Scheduled for /)).toHaveAttribute('title', '2026-11-18T15:00:00Z');
        expect(screen.getByText('0/2')).toBeInTheDocument();
        fireEvent.click(screen.getByTestId('quest-row'));
        expect(mockNavigate).toHaveBeenCalledWith('/tasks/quest-1');
    });

    it('keeps event and manual Quests as separate rows without an occurrence label', () => {
        mockRows.current['event-1'] = questResponse('event-1', null, 1);
        mockRows.current['manual-1'] = questResponse('manual-1', null, 1);

        render(<QuestsTable questIds={['event-1', 'manual-1']} />);

        expect(screen.getAllByTestId('quest-row')).toHaveLength(2);
        expect(screen.queryByText(/Scheduled for/)).not.toBeInTheDocument();
    });
});
