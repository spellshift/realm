import { describe, it, expect, beforeEach, vi } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { FilterProvider, useFilters, Filters, FilterFieldType, calculateFilterCount, calculateTotalFilterCount } from './../FilterContext';
import { FilterBarOption } from '../../../utils/interfacesUI';
import { MemoryRouter } from 'react-router-dom';
import React from 'react';

const STORAGE_KEY = 'realm-filters-v1.1';

const createWrapper = (initialEntries: string[] = ['/']) => {
  return function Wrapper({ children }: { children: React.ReactNode }) {
    return (
      <MemoryRouter initialEntries={initialEntries}>
        <FilterProvider>{children}</FilterProvider>
      </MemoryRouter>
    );
  };
};

describe('FilterContext', () => {
  beforeEach(() => {
    sessionStorage.clear();
    vi.clearAllMocks();
  });

  describe('FilterProvider initialization', () => {
    it('should provide default filters on initial load', () => {
      const { result } = renderHook(() => useFilters(), {
        wrapper: createWrapper(),
      });

      expect(result.current.filters).toEqual({
        isLocked: false,
        questName: '',
        taskOutput: '',
        beaconFields: [],
        tomeFields: [],
        tomeMultiSearch: '',
        assetName: '',
        userId: '',
      });
    });

    it('should load locked filters from sessionStorage', () => {
      const storedFilters: Filters = {
        isLocked: true,
        questName: 'test-quest',
        taskOutput: 'test-output',
        beaconFields: [{ kind: 'beacon', id: '1', name: 'Beacon 1' }],
        tomeFields: [],
        tomeMultiSearch: 'search-term',
        assetName: '',
        userId: '',
      };

      sessionStorage.setItem(STORAGE_KEY, JSON.stringify(storedFilters));

      const { result } = renderHook(() => useFilters(), {
        wrapper: createWrapper(),
      });

      expect(result.current.filters).toEqual(storedFilters);
    });

    it('should NOT load unlocked filters from sessionStorage', () => {
      const storedFilters: Filters = {
        isLocked: false,
        questName: 'test-quest',
        taskOutput: 'test-output',
        beaconFields: [{ kind: 'beacon', id: '1', name: 'Beacon 1' }],
        tomeFields: [],
        tomeMultiSearch: 'search-term',
        assetName: '',
        userId: '',
      };

      sessionStorage.setItem(STORAGE_KEY, JSON.stringify(storedFilters));

      const { result } = renderHook(() => useFilters(), {
        wrapper: createWrapper(),
      });

      expect(result.current.filters).toEqual({
        isLocked: false,
        questName: '',
        taskOutput: '',
        beaconFields: [],
        tomeFields: [],
        tomeMultiSearch: '',
        assetName: '',
        userId: '',
      });
    });

    it('should return default context values when useFilters is used outside FilterProvider', () => {
      // Note: The context provides default values, so it won't throw when used outside provider
      // but will return no-op functions and default filter state
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <MemoryRouter>{children}</MemoryRouter>
      );

      const { result } = renderHook(() => useFilters(), { wrapper });

      expect(result.current.filters).toEqual({
        isLocked: false,
        questName: '',
        taskOutput: '',
        beaconFields: [],
        tomeFields: [],
        tomeMultiSearch: '',
        assetName: '',
        userId: '',
      });
      expect(result.current.filterCount).toBe(0);
    });
  });

  describe('Filter validation', () => {
    it('should return default filters if sessionStorage contains invalid JSON', () => {
      sessionStorage.setItem(STORAGE_KEY, 'invalid-json');

      const { result } = renderHook(() => useFilters(), {
        wrapper: createWrapper(),
      });

      expect(result.current.filters).toEqual({
        isLocked: false,
        questName: '',
        taskOutput: '',
        beaconFields: [],
        tomeFields: [],
        tomeMultiSearch: '',
        assetName: '',
        userId: '',
      });
    });

    it('should reject data with invalid field types', () => {
      const invalidFilters = {
        isLocked: 'abc',
        questName: '',
        taskOutput: '',
        beaconFields: [],
        tomeFields: [],
        tomeMultiSearch: '',
      };

      sessionStorage.setItem(STORAGE_KEY, JSON.stringify(invalidFilters));

      const { result } = renderHook(() => useFilters(), {
        wrapper: createWrapper(),
      });

      expect(result.current.filters.isLocked).toBe(false);
    });

    it('should reject beaconFields with missing required fields', () => {
      const invalidFilters = {
        isLocked: true,
        questName: '',
        taskOutput: '',
        beaconFields: [{ kind: 'beacon', id: '1' }], // Missing 'name'
        tomeFields: [],
        tomeMultiSearch: '',
      };

      sessionStorage.setItem(STORAGE_KEY, JSON.stringify(invalidFilters));

      const { result } = renderHook(() => useFilters(), {
        wrapper: createWrapper(),
      });

      expect(result.current.filters.beaconFields).toEqual([]);
    });

    it('should accept valid FilterBarOption with optional fields when locked', () => {
      const validFilters: Filters = {
        isLocked: true,
        questName: '',
        taskOutput: '',
        beaconFields: [
          { kind: 'beacon', id: '1', name: 'Beacon 1', label: 'Label 1', value: 'value-1' },
        ],
        tomeFields: [],
        tomeMultiSearch: '',
        assetName: '',
        userId: '',
      };

      sessionStorage.setItem(STORAGE_KEY, JSON.stringify(validFilters));

      const { result } = renderHook(() => useFilters(), {
        wrapper: createWrapper(),
      });

      expect(result.current.filters.beaconFields).toEqual(validFilters.beaconFields);
    });
  });

  describe('updateFilters', () => {
    it('should update filters with partial updates', () => {
      const { result } = renderHook(() => useFilters(), {
        wrapper: createWrapper(),
      });

      act(() => {
        result.current.updateFilters({ questName: 'new-quest' });
      });

      expect(result.current.filters.questName).toBe('new-quest');
      expect(result.current.filters.isLocked).toBe(false);
    });

    it('should update multiple filter fields at once', () => {
      const { result } = renderHook(() => useFilters(), {
        wrapper: createWrapper(),
      });

      act(() => {
        result.current.updateFilters({
          questName: 'multi-quest',
          taskOutput: 'multi-output',
        });
      });

      expect(result.current.filters.questName).toBe('multi-quest');
      expect(result.current.filters.taskOutput).toBe('multi-output');
      expect(result.current.filters.isLocked).toBe(false);
    });

    it('should persist updated filters to sessionStorage', async () => {
      const { result } = renderHook(() => useFilters(), {
        wrapper: createWrapper(),
      });

      act(() => {
        result.current.updateFilters({ questName: 'stored-quest' });
      });

      await waitFor(() => {
        const stored = sessionStorage.getItem(STORAGE_KEY);
        expect(stored).toBeTruthy();
        const parsed = JSON.parse(stored!);
        expect(parsed.questName).toBe('stored-quest');
      });
    });

    it('should update array fields', () => {
      const { result } = renderHook(() => useFilters(), {
        wrapper: createWrapper(),
      });

      const newBeaconFields: FilterBarOption[] = [
        { kind: 'beacon', id: '1', name: 'Beacon 1' },
        { kind: 'beacon', id: '2', name: 'Beacon 2' },
      ];

      const newTomeFields: FilterBarOption[] = [
        { kind: 'tome', id: 't1', name: 'Tome 1' },
      ];

      act(() => {
        result.current.updateFilters({
          beaconFields: newBeaconFields,
          tomeFields: newTomeFields
        });
      });

      expect(result.current.filters.beaconFields).toEqual(newBeaconFields);
      expect(result.current.filters.tomeFields).toEqual(newTomeFields);
    });
  });

  describe('resetFilters', () => {
    it('should reset all filters to default values', () => {
      const { result } = renderHook(() => useFilters(), {
        wrapper: createWrapper(),
      });

      act(() => {
        result.current.updateFilters({
          questName: 'custom-quest',
          taskOutput: 'custom-output',
          isLocked: false,
          beaconFields: [{ kind: 'beacon', id: '1', name: 'Beacon 1' }],
        });
      });

      act(() => {
        result.current.resetFilters();
      });

      expect(result.current.filters).toEqual({
        isLocked: false,
        questName: '',
        taskOutput: '',
        beaconFields: [],
        tomeFields: [],
        tomeMultiSearch: '',
        assetName: '',
        userId: '',
      });
    });

    it('should persist default filters to sessionStorage after reset', async () => {
      const { result } = renderHook(() => useFilters(), {
        wrapper: createWrapper(),
      });

      act(() => {
        result.current.updateFilters({ questName: 'to-be-removed' });
      });

      act(() => {
        result.current.resetFilters();
      });

      await waitFor(() => {
        const stored = sessionStorage.getItem(STORAGE_KEY);
        expect(stored).toBeTruthy();
        const parsed = JSON.parse(stored!);
        expect(parsed).toEqual({
          isLocked: false,
          questName: '',
          taskOutput: '',
          beaconFields: [],
          tomeFields: [],
          tomeMultiSearch: '',
          assetName: '',
          userId: '',
        });
      });
    });
  });

  describe('storage event listener', () => {
    it('should update filters when storage event is triggered with locked filters', () => {
      const { result } = renderHook(() => useFilters(), {
        wrapper: createWrapper(),
      });

      const newFilters: Filters = {
        isLocked: true,
        questName: 'external-update',
        taskOutput: 'external-output',
        beaconFields: [],
        tomeFields: [],
        tomeMultiSearch: 'external-search',
        assetName: '',
        userId: '',
      };

      act(() => {
        const storageEvent = new StorageEvent('storage', {
          key: STORAGE_KEY,
          newValue: JSON.stringify(newFilters),
        });
        window.dispatchEvent(storageEvent);
      });

      expect(result.current.filters).toEqual(newFilters);
    });

    it('should reset to defaults when storage event contains unlocked filters', () => {
      const { result } = renderHook(() => useFilters(), {
        wrapper: createWrapper(),
      });

      // First set some filters
      act(() => {
        result.current.updateFilters({ questName: 'initial-quest', isLocked: true });
      });

      const unlockedFilters: Filters = {
        isLocked: false,
        questName: 'external-update',
        taskOutput: 'external-output',
        beaconFields: [],
        tomeFields: [],
        tomeMultiSearch: 'external-search',
        assetName: '',
        userId: '',
      };

      act(() => {
        const storageEvent = new StorageEvent('storage', {
          key: STORAGE_KEY,
          newValue: JSON.stringify(unlockedFilters),
        });
        window.dispatchEvent(storageEvent);
      });

      // Should reset to defaults since isLocked is false
      expect(result.current.filters).toEqual({
        isLocked: false,
        questName: '',
        taskOutput: '',
        beaconFields: [],
        tomeFields: [],
        tomeMultiSearch: '',
        assetName: '',
        userId: '',
      });
    });

    it('should ignore storage events with different keys', () => {
      const { result } = renderHook(() => useFilters(), {
        wrapper: createWrapper(),
      });

      const initialFilters = result.current.filters;

      act(() => {
        const storageEvent = new StorageEvent('storage', {
          key: 'different-key',
          newValue: JSON.stringify({ some: 'data' }),
        });
        window.dispatchEvent(storageEvent);
      });

      expect(result.current.filters).toEqual(initialFilters);
    });
  });

  describe('calculateFilterCount', () => {
    const baseFilters: Filters = {
      isLocked: false,
      questName: '',
      taskOutput: '',
      beaconFields: [],
      tomeFields: [],
      tomeMultiSearch: '',
      assetName: '',
      userId: '',
    };

    it('should return 1 for non-empty questName', () => {
      const filters = { ...baseFilters, questName: 'test' };
      expect(calculateFilterCount(filters, FilterFieldType.QUEST_NAME)).toBe(1);
    });

    it('should return 0 for empty questName', () => {
      expect(calculateFilterCount(baseFilters, FilterFieldType.QUEST_NAME)).toBe(0);
    });

    it('should return 1 for non-empty taskOutput', () => {
      const filters = { ...baseFilters, taskOutput: 'output' };
      expect(calculateFilterCount(filters, FilterFieldType.TASK_OUTPUT)).toBe(1);
    });

    it('should return 0 for empty taskOutput', () => {
      expect(calculateFilterCount(baseFilters, FilterFieldType.TASK_OUTPUT)).toBe(0);
    });

    it('should return 1 for non-empty tomeMultiSearch', () => {
      const filters = { ...baseFilters, tomeMultiSearch: 'search' };
      expect(calculateFilterCount(filters, FilterFieldType.TOME_MULTI_SEARCH)).toBe(1);
    });

    it('should return 0 for empty tomeMultiSearch', () => {
      expect(calculateFilterCount(baseFilters, FilterFieldType.TOME_MULTI_SEARCH)).toBe(0);
    });

    it('should return array length for beaconFields', () => {
      const filters = {
        ...baseFilters,
        beaconFields: [
          { kind: 'beacon', id: '1', name: 'B1' },
          { kind: 'beacon', id: '2', name: 'B2' },
          { kind: 'beacon', id: '3', name: 'B3' },
        ],
      };
      expect(calculateFilterCount(filters, FilterFieldType.BEACON_FIELDS)).toBe(3);
    });

    it('should return array length for tomeFields', () => {
      const filters = {
        ...baseFilters,
        tomeFields: [
          { kind: 'tome', id: '1', name: 'T1' },
        ],
      };
      expect(calculateFilterCount(filters, FilterFieldType.TOME_FIELDS)).toBe(1);
    });
  });

  describe('calculateTotalFilterCount', () => {
    const baseFilters: Filters = {
      isLocked: false,
      questName: '',
      taskOutput: '',
      beaconFields: [],
      tomeFields: [],
      tomeMultiSearch: '',
      assetName: '',
      userId: '',
    };

    it('should return 0 when all filters are empty', () => {
      const allFields = Object.values(FilterFieldType);
      expect(calculateTotalFilterCount(baseFilters, allFields)).toBe(0);
    });

    it('should sum counts for specified fields only', () => {
      const filters: Filters = {
        ...baseFilters,
        questName: 'test',
        taskOutput: 'output',
        beaconFields: [{ kind: 'beacon', id: '1', name: 'B1' }],
      };

      // Only count questName and taskOutput
      expect(calculateTotalFilterCount(filters, [
        FilterFieldType.QUEST_NAME,
        FilterFieldType.TASK_OUTPUT,
      ])).toBe(2);
    });

    it('should return total count for all fields', () => {
      const filters: Filters = {
        isLocked: false,
        questName: 'test',
        taskOutput: 'output',
        tomeMultiSearch: 'search',
        beaconFields: [
          { kind: 'beacon', id: '1', name: 'B1' },
          { kind: 'beacon', id: '2', name: 'B2' },
        ],
        tomeFields: [
          { kind: 'tome', id: '1', name: 'T1' },
        ],
        assetName: '',
        userId: '',
      };

      const allFields = Object.values(FilterFieldType);
      // 1 + 1 + 1 + 2 + 1 = 6
      expect(calculateTotalFilterCount(filters, allFields)).toBe(6);
    });
  });
});
