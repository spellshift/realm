import { describe, it, expect, beforeEach, vi } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { FilterProvider, useFilters, Filters } from './../FilterContext';
import { FilterBarOption } from '../../../utils/interfacesUI';

const STORAGE_KEY = 'realm-filters-v1.1';

describe('FilterContext', () => {
  beforeEach(() => {
    sessionStorage.clear();
    vi.clearAllMocks();
  });

  describe('FilterProvider initialization', () => {
    it('should provide default filters on initial load', () => {
      const { result } = renderHook(() => useFilters(), {
        wrapper: FilterProvider,
      });

      expect(result.current.filters).toEqual({
        filtersEnabled: true,
        questName: '',
        taskOutput: '',
        beaconFields: [],
        tomeFields: [],
        tomeMultiSearch: '',
      });
    });

    it('should load filters from sessionStorage if available', () => {
      const storedFilters: Filters = {
        filtersEnabled: false,
        questName: 'test-quest',
        taskOutput: 'test-output',
        beaconFields: [{ kind: 'beacon', id: '1', name: 'Beacon 1' }],
        tomeFields: [],
        tomeMultiSearch: 'search-term',
      };

      sessionStorage.setItem(STORAGE_KEY, JSON.stringify(storedFilters));

      const { result } = renderHook(() => useFilters(), {
        wrapper: FilterProvider,
      });

      expect(result.current.filters).toEqual(storedFilters);
    });

    it('should throw error when useFilters is used outside FilterProvider', () => {
      const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});

      expect(() => {
        renderHook(() => useFilters());
      }).toThrow('useFilters must be used within a FilterProvider');

      consoleError.mockRestore();
    });
  });

  describe('Filter validation', () => {
    it('should return default filters if sessionStorage contains invalid JSON', () => {
      sessionStorage.setItem(STORAGE_KEY, 'invalid-json');

      const { result } = renderHook(() => useFilters(), {
        wrapper: FilterProvider,
      });

      expect(result.current.filters).toEqual({
        filtersEnabled: true,
        questName: '',
        taskOutput: '',
        beaconFields: [],
        tomeFields: [],
        tomeMultiSearch: '',
      });
    });

    it('should reject data with invalid field types', () => {
      const invalidFilters = {
        filtersEnabled: 'not-a-boolean',
        questName: '',
        taskOutput: '',
        beaconFields: [],
        tomeFields: [],
        tomeMultiSearch: '',
      };

      sessionStorage.setItem(STORAGE_KEY, JSON.stringify(invalidFilters));

      const { result } = renderHook(() => useFilters(), {
        wrapper: FilterProvider,
      });

      expect(result.current.filters.filtersEnabled).toBe(true);
    });

    it('should reject beaconFields with missing required fields', () => {
      const invalidFilters = {
        filtersEnabled: true,
        questName: '',
        taskOutput: '',
        beaconFields: [{ kind: 'beacon', id: '1' }], // Missing 'name'
        tomeFields: [],
        tomeMultiSearch: '',
      };

      sessionStorage.setItem(STORAGE_KEY, JSON.stringify(invalidFilters));

      const { result } = renderHook(() => useFilters(), {
        wrapper: FilterProvider,
      });

      expect(result.current.filters.beaconFields).toEqual([]);
    });

    it('should accept valid FilterBarOption with optional fields', () => {
      const validFilters: Filters = {
        filtersEnabled: true,
        questName: '',
        taskOutput: '',
        beaconFields: [
          { kind: 'beacon', id: '1', name: 'Beacon 1', label: 'Label 1', value: 'value-1' },
        ],
        tomeFields: [],
        tomeMultiSearch: '',
      };

      sessionStorage.setItem(STORAGE_KEY, JSON.stringify(validFilters));

      const { result } = renderHook(() => useFilters(), {
        wrapper: FilterProvider,
      });

      expect(result.current.filters.beaconFields).toEqual(validFilters.beaconFields);
    });
  });

  describe('updateFilters', () => {
    it('should update filters with partial updates', () => {
      const { result } = renderHook(() => useFilters(), {
        wrapper: FilterProvider,
      });

      act(() => {
        result.current.updateFilters({ questName: 'new-quest' });
      });

      expect(result.current.filters.questName).toBe('new-quest');
      expect(result.current.filters.filtersEnabled).toBe(true);
    });

    it('should update multiple filter fields at once', () => {
      const { result } = renderHook(() => useFilters(), {
        wrapper: FilterProvider,
      });

      act(() => {
        result.current.updateFilters({
          questName: 'multi-quest',
          taskOutput: 'multi-output',
          filtersEnabled: false,
        });
      });

      expect(result.current.filters.questName).toBe('multi-quest');
      expect(result.current.filters.taskOutput).toBe('multi-output');
      expect(result.current.filters.filtersEnabled).toBe(false);
    });

    it('should persist updated filters to sessionStorage', async () => {
      const { result } = renderHook(() => useFilters(), {
        wrapper: FilterProvider,
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
        wrapper: FilterProvider,
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
        wrapper: FilterProvider,
      });

      act(() => {
        result.current.updateFilters({
          questName: 'custom-quest',
          taskOutput: 'custom-output',
          filtersEnabled: false,
          beaconFields: [{ kind: 'beacon', id: '1', name: 'Beacon 1' }],
        });
      });

      act(() => {
        result.current.resetFilters();
      });

      expect(result.current.filters).toEqual({
        filtersEnabled: true,
        questName: '',
        taskOutput: '',
        beaconFields: [],
        tomeFields: [],
        tomeMultiSearch: '',
      });
    });

    it('should persist default filters to sessionStorage after reset', async () => {
      const { result } = renderHook(() => useFilters(), {
        wrapper: FilterProvider,
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
          filtersEnabled: true,
          questName: '',
          taskOutput: '',
          beaconFields: [],
          tomeFields: [],
          tomeMultiSearch: '',
        });
      });
    });
  });

  describe('storage event listener', () => {
    it('should update filters when storage event is triggered', () => {
      const { result } = renderHook(() => useFilters(), {
        wrapper: FilterProvider,
      });

      const newFilters: Filters = {
        filtersEnabled: false,
        questName: 'external-update',
        taskOutput: 'external-output',
        beaconFields: [],
        tomeFields: [],
        tomeMultiSearch: 'external-search',
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

    it('should ignore storage events with different keys', () => {
      const { result } = renderHook(() => useFilters(), {
        wrapper: FilterProvider,
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
});
