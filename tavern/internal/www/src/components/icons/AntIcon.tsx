// lucide has no ant glyph, so this is hand-drawn in lucide's style:
// 24x24 viewBox, 2px round-capped strokes, currentColor.
export const AntIcon = ({ className }: { className?: string }) => (
    <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth={2}
        strokeLinecap="round"
        strokeLinejoin="round"
        className={className}
        aria-hidden="true"
    >
        <circle cx="12" cy="5.5" r="2" />
        <ellipse cx="12" cy="11.5" rx="1.6" ry="2" />
        <ellipse cx="12" cy="18.2" rx="2.6" ry="3.3" />
        <path d="M11 3.8 9 2M13 3.8 15 2" />
        <path d="M10.5 10 5 7.5M10.4 11.5 4 12M10.6 13 5 16.5" />
        <path d="M13.5 10 19 7.5M13.6 11.5 20 12M13.4 13 19 16.5" />
    </svg>
);

export default AntIcon;
