import { tv } from 'tailwind-variants';

export const baseBadge = tv({
  base: 'py-1 px-2 rounded text-xs font-semibold',
});

// create solid Badge styles
export const solidBadge = tv({
  extend: baseBadge,
  variants: {
    color: {
        none: "text-gray-800",
        purple:
        'btn-primary',
        red: 'bg-red-600 text-white',
        gray: 'bg-gray-200 text-gray-900',
        green: ' bg-green-600 text-white'
    },
  },
});

//create outline Badge styles
export const outlineBadge = tv({
  extend: baseBadge,
  base: 'ring-1',
  variants: {
    color: {
      none: "text-gray-800 ring-gray-600",
      purple: 'text-purple-800 ring-purple-800',
      red: 'text-red-600 ring-red-800',
      gray: 'text-gray-900 ring-gray-500',
      green: ' bg-green-600 text-white'
    },
  },
});
