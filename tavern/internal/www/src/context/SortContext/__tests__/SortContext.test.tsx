import { describe, it, expect, beforeEach, vi } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { SortsProvider, useSorts } from '../SortContext';
import { AssetOrderField, HostOrderField, OrderDirection, PageNavItem, ProcessOrderField, QuestOrderField, TaskOrderField } from '../../../utils/enums';
import { OrderByField } from '../../../utils/interfacesQuery';
import { Sorts } from '../sortingUtils';

const STORAGE_KEY = 'realm-sorting-v1.0';

describe('SortContext', () => {
  beforeEach(() => {
    sessionStorage.clear();
    vi.clearAllMocks();
  });

  describe('SortsProvider initialization', () => {
    it('should provide default sorts on initial load', () => {
      const { result } = renderHook(() => useSorts(), {
        wrapper: SortsProvider,
      });

      expect(result.current.sorts).toEqual({
        [PageNavItem.hosts]: {
          direction: OrderDirection.Desc,
          field: HostOrderField.CreatedAt,
        },
        [PageNavItem.quests]: {
          direction: OrderDirection.Desc,
          field: QuestOrderField.CreatedAt,
        },
        [PageNavItem.tasks]: {
          direction: OrderDirection.Desc,
          field: TaskOrderField.LastModifiedAt,
        },
        [PageNavItem.assets]: {
          direction: OrderDirection.Desc,
          field: AssetOrderField.CreatedAt,
        },
        [PageNavItem.processes]: {
          direction: OrderDirection.Desc,
          field: ProcessOrderField.LastModifiedAt,
        },
      });
    });

    it('should load sorts from sessionStorage if available', () => {
      const storedSorts: Sorts = {
        [PageNavItem.hosts]: {
          direction: OrderDirection.Asc,
          field: HostOrderField.LastSeenAt,
        },
        [PageNavItem.quests]: {
          direction: OrderDirection.Asc,
          field: QuestOrderField.Name,
        },
        [PageNavItem.tasks]: {
          direction: OrderDirection.Asc,
          field: TaskOrderField.CreatedAt,
        },
        [PageNavItem.assets]: {
          direction: OrderDirection.Desc,
          field: AssetOrderField.CreatedAt,
        },
        [PageNavItem.processes]: {
          direction: OrderDirection.Desc,
          field: ProcessOrderField.LastModifiedAt,
        },
      };

      sessionStorage.setItem(STORAGE_KEY, JSON.stringify(storedSorts));

      const { result } = renderHook(() => useSorts(), {
        wrapper: SortsProvider,
      });

      expect(result.current.sorts).toEqual(storedSorts);
    });

    it('should throw error when useSorts is used outside SortsProvider', () => {
      const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});

      expect(() => {
        renderHook(() => useSorts());
      }).toThrow('useSorts must be used within a SortProvider');

      consoleError.mockRestore();
    });
  });

  describe('Sort validation', () => {
    it('should return default sorts if sessionStorage contains invalid JSON', () => {
      sessionStorage.setItem(STORAGE_KEY, 'invalid-json');

      const { result } = renderHook(() => useSorts(), {
        wrapper: SortsProvider,
      });

      expect(result.current.sorts[PageNavItem.hosts]).toEqual({
        direction: OrderDirection.Desc,
        field: HostOrderField.CreatedAt,
      });
    });

    it('should reject data with missing required fields', () => {
      const invalidSorts = {
        [PageNavItem.hosts]: {
          direction: OrderDirection.Desc,
          // Missing 'field'
        },
        [PageNavItem.quests]: {
          direction: OrderDirection.Desc,
          field: QuestOrderField.CreatedAt,
        },
        [PageNavItem.tasks]: {
          direction: OrderDirection.Desc,
          field: TaskOrderField.LastModifiedAt,
        },
      };

      sessionStorage.setItem(STORAGE_KEY, JSON.stringify(invalidSorts));

      const { result } = renderHook(() => useSorts(), {
        wrapper: SortsProvider,
      });

      expect(result.current.sorts[PageNavItem.hosts].field).toBe(HostOrderField.CreatedAt);
    });

    it('should reject data with invalid direction value', () => {
      const invalidSorts = {
        [PageNavItem.hosts]: {
          direction: 'INVALID_DIRECTION',
          field: HostOrderField.CreatedAt,
        },
        [PageNavItem.quests]: {
          direction: OrderDirection.Desc,
          field: QuestOrderField.CreatedAt,
        },
        [PageNavItem.tasks]: {
          direction: OrderDirection.Desc,
          field: TaskOrderField.LastModifiedAt,
        },
      };

      sessionStorage.setItem(STORAGE_KEY, JSON.stringify(invalidSorts));

      const { result } = renderHook(() => useSorts(), {
        wrapper: SortsProvider,
      });

      expect(result.current.sorts[PageNavItem.hosts].direction).toBe(OrderDirection.Desc);
    });

    it('should accept valid OrderByField structure', () => {
      const validSorts: Sorts = {
        [PageNavItem.hosts]: {
          direction: OrderDirection.Asc,
          field: HostOrderField.LastSeenAt,
        },
        [PageNavItem.quests]: {
          direction: OrderDirection.Asc,
          field: QuestOrderField.Name,
        },
        [PageNavItem.tasks]: {
          direction: OrderDirection.Asc,
          field: TaskOrderField.CreatedAt,
        },
        [PageNavItem.assets]: {
          direction: OrderDirection.Desc,
          field: AssetOrderField.CreatedAt,
        },
        [PageNavItem.processes]: {
          direction: OrderDirection.Desc,
          field: ProcessOrderField.LastModifiedAt,
        },
      };

      sessionStorage.setItem(STORAGE_KEY, JSON.stringify(validSorts));

      const { result } = renderHook(() => useSorts(), {
        wrapper: SortsProvider,
      });

      expect(result.current.sorts).toEqual(validSorts);
    });
  });

  describe('updateSorts', () => {
    it('should update sorts with partial updates', () => {
      const { result } = renderHook(() => useSorts(), {
        wrapper: SortsProvider,
      });

      const newHostSort: OrderByField = {
        direction: OrderDirection.Asc,
        field: HostOrderField.LastSeenAt,
      };

      act(() => {
        result.current.updateSorts({ [PageNavItem.hosts]: newHostSort });
      });

      expect(result.current.sorts[PageNavItem.hosts]).toEqual(newHostSort);
      expect(result.current.sorts[PageNavItem.quests].field).toBe(QuestOrderField.CreatedAt);
    });

    it('should update multiple sort fields at once', () => {
      const { result } = renderHook(() => useSorts(), {
        wrapper: SortsProvider,
      });

      act(() => {
        result.current.updateSorts({
          [PageNavItem.hosts]: {
            direction: OrderDirection.Asc,
            field: HostOrderField.LastSeenAt,
          },
          [PageNavItem.quests]: {
            direction: OrderDirection.Asc,
            field: QuestOrderField.Name,
          },
        });
      });

      expect(result.current.sorts[PageNavItem.hosts]).toEqual({
        direction: OrderDirection.Asc,
        field: HostOrderField.LastSeenAt,
      });
      expect(result.current.sorts[PageNavItem.quests]).toEqual({
        direction: OrderDirection.Asc,
        field: QuestOrderField.Name,
      });
    });

    it('should persist updated sorts to sessionStorage', async () => {
      const { result } = renderHook(() => useSorts(), {
        wrapper: SortsProvider,
      });

      act(() => {
        result.current.updateSorts({
          [PageNavItem.hosts]: {
            direction: OrderDirection.Asc,
            field: HostOrderField.LastSeenAt,
          },
        });
      });

      await waitFor(() => {
        const stored = sessionStorage.getItem(STORAGE_KEY);
        expect(stored).toBeTruthy();
        const parsed = JSON.parse(stored!);
        expect(parsed[PageNavItem.hosts]).toEqual({
          direction: OrderDirection.Asc,
          field: HostOrderField.LastSeenAt,
        });
      });
    });
  });

  describe('resetSorts', () => {
    it('should reset all sorts to default values', () => {
      const { result } = renderHook(() => useSorts(), {
        wrapper: SortsProvider,
      });

      act(() => {
        result.current.updateSorts({
          [PageNavItem.hosts]: {
            direction: OrderDirection.Asc,
            field: HostOrderField.LastSeenAt,
          },
        });
      });

      act(() => {
        result.current.resetSorts();
      });

      expect(result.current.sorts).toEqual({
        [PageNavItem.hosts]: {
          direction: OrderDirection.Desc,
          field: HostOrderField.CreatedAt,
        },
        [PageNavItem.quests]: {
          direction: OrderDirection.Desc,
          field: QuestOrderField.CreatedAt,
        },
        [PageNavItem.tasks]: {
          direction: OrderDirection.Desc,
          field: TaskOrderField.LastModifiedAt,
        },
        [PageNavItem.assets]: {
          direction: OrderDirection.Desc,
          field: AssetOrderField.CreatedAt,
        },
        [PageNavItem.processes]: {
          direction: OrderDirection.Desc,
          field: ProcessOrderField.LastModifiedAt,
        },
      });
    });

    it('should persist default sorts to sessionStorage after reset', async () => {
      const { result } = renderHook(() => useSorts(), {
        wrapper: SortsProvider,
      });

      act(() => {
        result.current.updateSorts({
          [PageNavItem.hosts]: {
            direction: OrderDirection.Asc,
            field: HostOrderField.LastSeenAt,
          },
        });
      });

      act(() => {
        result.current.resetSorts();
      });

      await waitFor(() => {
        const stored = sessionStorage.getItem(STORAGE_KEY);
        expect(stored).toBeTruthy();
        const parsed = JSON.parse(stored!);
        expect(parsed).toEqual({
          [PageNavItem.hosts]: {
            direction: OrderDirection.Desc,
            field: HostOrderField.CreatedAt,
          },
          [PageNavItem.quests]: {
            direction: OrderDirection.Desc,
            field: QuestOrderField.CreatedAt,
          },
          [PageNavItem.tasks]: {
            direction: OrderDirection.Desc,
            field: TaskOrderField.LastModifiedAt,
          },
          [PageNavItem.assets]: {
            direction: OrderDirection.Desc,
            field: AssetOrderField.CreatedAt,
          },
          [PageNavItem.processes]: {
            direction: OrderDirection.Desc,
            field: ProcessOrderField.LastModifiedAt,
          },
        });
      });
    });
  });

  describe('storage event listener', () => {
    it('should update sorts when storage event is triggered', () => {
      const { result } = renderHook(() => useSorts(), {
        wrapper: SortsProvider,
      });

      const newSorts: Sorts = {
        [PageNavItem.hosts]: {
          direction: OrderDirection.Asc,
          field: HostOrderField.LastSeenAt,
        },
        [PageNavItem.quests]: {
          direction: OrderDirection.Asc,
          field: QuestOrderField.Name,
        },
        [PageNavItem.tasks]: {
          direction: OrderDirection.Asc,
          field: TaskOrderField.CreatedAt,
        },
        [PageNavItem.assets]: {
          direction: OrderDirection.Desc,
          field: AssetOrderField.CreatedAt,
        },
        [PageNavItem.processes]: {
          direction: OrderDirection.Desc,
          field: ProcessOrderField.LastModifiedAt,
        },
      };

      act(() => {
        const storageEvent = new StorageEvent('storage', {
          key: STORAGE_KEY,
          newValue: JSON.stringify(newSorts),
        });
        window.dispatchEvent(storageEvent);
      });

      expect(result.current.sorts).toEqual(newSorts);
    });

    it('should ignore storage events with different keys', () => {
      const { result } = renderHook(() => useSorts(), {
        wrapper: SortsProvider,
      });

      const initialSorts = result.current.sorts;

      act(() => {
        const storageEvent = new StorageEvent('storage', {
          key: 'different-key',
          newValue: JSON.stringify({ some: 'data' }),
        });
        window.dispatchEvent(storageEvent);
      });

      expect(result.current.sorts).toEqual(initialSorts);
    });
  });
});
