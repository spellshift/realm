import { describe, it, expect, beforeEach, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { CreateQuestModalProvider, useCreateQuestModal } from '../CreateQuestModalContext';
import { MemoryRouter } from 'react-router-dom';
import React from 'react';

const mockNavigate = vi.fn();

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  };
});

vi.mock('../../../components/create-quest-modal', () => ({
  CreateQuestModal: vi.fn(({ setOpen, onComplete }: {
    isOpen: boolean;
    setOpen: (open: boolean) => void;
    onComplete: (questId: string) => void;
  }) => (
    <div data-testid="mock-create-quest-modal">
      <button data-testid="close-button" onClick={() => setOpen(false)}>Close</button>
      <button data-testid="complete-button" onClick={() => onComplete('test-quest-id')}>Complete</button>
    </div>
  )),
}));

const createWrapper = (initialEntries: string[] = ['/']) => {
  return function Wrapper({ children }: { children: React.ReactNode }) {
    return (
      <MemoryRouter initialEntries={initialEntries}>
        <CreateQuestModalProvider>{children}</CreateQuestModalProvider>
      </MemoryRouter>
    );
  };
};

describe('CreateQuestModalContext', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('CreateQuestModalProvider initialization', () => {
    it('should provide default closed state on initial load', () => {
      const { result } = renderHook(() => useCreateQuestModal(), {
        wrapper: createWrapper(),
      });

      expect(result.current.isOpen).toBe(false);
      expect(typeof result.current.openModal).toBe('function');
      expect(typeof result.current.closeModal).toBe('function');
    });
  });

  describe('useCreateQuestModal hook', () => {
    it('should throw error when used outside provider', () => {
      expect(() => {
        renderHook(() => useCreateQuestModal());
      }).toThrow('useCreateQuestModal must be used within a CreateQuestModalProvider');
    });
  });

  describe('openModal', () => {
    it('should set isOpen to true when called', () => {
      const { result } = renderHook(() => useCreateQuestModal(), {
        wrapper: createWrapper(),
      });

      expect(result.current.isOpen).toBe(false);

      act(() => {
        result.current.openModal();
      });

      expect(result.current.isOpen).toBe(true);
    });

    it('should accept options when opening modal', () => {
      const { result } = renderHook(() => useCreateQuestModal(), {
        wrapper: createWrapper(),
      });

      const mockOnComplete = vi.fn();

      act(() => {
        result.current.openModal({
          initialFormData: { name: 'Test Quest' },
          onComplete: mockOnComplete,
          navigateToQuest: true,
        });
      });

      expect(result.current.isOpen).toBe(true);
    });
  });

  describe('closeModal', () => {
    it('should set isOpen to false when called', () => {
      const { result } = renderHook(() => useCreateQuestModal(), {
        wrapper: createWrapper(),
      });

      act(() => {
        result.current.openModal();
      });

      expect(result.current.isOpen).toBe(true);

      act(() => {
        result.current.closeModal();
      });

      expect(result.current.isOpen).toBe(false);
    });
  });

  describe('modal open/close cycle', () => {
    it('should handle multiple open/close cycles', () => {
      const { result } = renderHook(() => useCreateQuestModal(), {
        wrapper: createWrapper(),
      });

      // First cycle
      act(() => {
        result.current.openModal();
      });
      expect(result.current.isOpen).toBe(true);

      act(() => {
        result.current.closeModal();
      });
      expect(result.current.isOpen).toBe(false);

      // Second cycle
      act(() => {
        result.current.openModal();
      });
      expect(result.current.isOpen).toBe(true);

      act(() => {
        result.current.closeModal();
      });
      expect(result.current.isOpen).toBe(false);
    });
  });

  describe('context value stability', () => {
    it('should provide stable function references', () => {
      const { result, rerender } = renderHook(() => useCreateQuestModal(), {
        wrapper: createWrapper(),
      });

      const initialOpenModal = result.current.openModal;
      const initialCloseModal = result.current.closeModal;

      rerender();

      expect(result.current.openModal).toBe(initialOpenModal);
      expect(result.current.closeModal).toBe(initialCloseModal);
    });
  });
});
