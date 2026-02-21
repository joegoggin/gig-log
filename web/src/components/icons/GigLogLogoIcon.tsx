type GigLogLogoIconProps = {
    /** Whether to render the GigLog wordmark text next to the logo mark. */
    showWordmark?: boolean;
    /** Whether to render the subtitle under the GigLog wordmark. */
    showSubtitle?: boolean;
};

/**
 * The GigLog brand logo with the mark and optional company wordmark.
 *
 * ## Props
 *
 * - `showWordmark` - Whether to render the GigLog wordmark text next to the logo mark
 * - `showSubtitle` - Whether to render the subtitle under the GigLog wordmark
 *
 * ## Example
 *
 * ```tsx
 * <GigLogLogoIcon showWordmark={false} />
 * ```
 */
const GigLogLogoIcon: React.FC<GigLogLogoIconProps> = ({
    showWordmark = true,
    showSubtitle = true,
}) => {
    const showSubtitleText = showWordmark && showSubtitle;

    const wordmarkWidth = 140;
    const textX = 48;

    return (
        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox={showWordmark ? (showSubtitleText ? `0 0 ${wordmarkWidth} 56` : `0 0 ${wordmarkWidth} 48`) : "0 0 48 48"}
            width={showWordmark ? `${wordmarkWidth}` : "48"}
            height={showWordmark ? (showSubtitleText ? "56" : "48") : "48"}
            fill="none"
        >
            <g className="giglog-logo__mark" transform="translate(2 2) scale(1.8)">
                <path
                    d="M7.53 2.5h10.83a1 1 0 0 1 .86 1.51l-1.65 2.84a1 1 0 0 1-.86.49H5.88a1 1 0 0 1-.87-1.49l1.66-2.85a1 1 0 0 1 .86-.5Z"
                    fill="rgb(var(--logo-mark-blue-rgb, 122, 162, 247))"
                />
                <path
                    d="M17.83 6.3h3.28a1 1 0 0 1 .86 1.5L16.6 17a1 1 0 0 1-.86.5h-3.28a1 1 0 0 1-.86-1.5l5.36-9.2a1 1 0 0 1 .87-.5Z"
                    fill="rgb(var(--logo-mark-cyan-rgb, 125, 207, 255))"
                />
                <path
                    d="M13.53 16.56h3.29a1 1 0 0 1 .86 1.49l-1.72 2.95a1 1 0 0 1-.86.5H4.27a1 1 0 0 1-.86-1.5l1.72-2.94a1 1 0 0 1 .86-.5Z"
                    fill="rgb(var(--logo-mark-yellow-rgb, 224, 175, 104))"
                />
                <path
                    d="M2.9 14.43 8.27 5.24a1 1 0 0 1 .86-.5h3.3a1 1 0 0 1 .85 1.5l-5.35 9.2a1 1 0 0 1-.87.5h-3.3a1 1 0 0 1-.86-1.5Z"
                    fill="rgb(var(--logo-mark-magenta-rgb, 187, 154, 247))"
                />
                <path
                    d="M2.03 13.05 3.75 10.1a1 1 0 0 1 .86-.5h3.28a1 1 0 0 1 .86 1.5l-1.72 2.95a1 1 0 0 1-.86.5H2.9a1 1 0 0 1-.87-1.5Z"
                    fill="rgb(var(--logo-mark-green-rgb, 158, 206, 106))"
                />
                <path
                    d="M11.57 10.24h8.75a1 1 0 0 1 .86 1.5l-1.72 2.95a1 1 0 0 1-.86.5h-8.76a1 1 0 0 1-.86-1.5l1.72-2.95a1 1 0 0 1 .87-.5Z"
                    fill="rgb(var(--logo-mark-green-rgb, 158, 206, 106))"
                />
                <path
                    d="M6.53 3.8h10.74"
                    stroke="rgb(var(--logo-mark-blue-highlight-rgb, 149, 181, 249))"
                    strokeWidth="0.8"
                    strokeLinecap="round"
                    opacity="0.8"
                />
                <path
                    d="M3.18 13.33h3.6"
                    stroke="rgb(var(--logo-mark-green-highlight-rgb, 197, 226, 166))"
                    strokeWidth="0.7"
                    strokeLinecap="round"
                    opacity="0.9"
                />
                <path
                    d="M4.3 20.12h10.72"
                    stroke="rgb(var(--logo-mark-yellow-highlight-rgb, 236, 207, 164))"
                    strokeWidth="0.8"
                    strokeLinecap="round"
                    opacity="0.7"
                />
            </g>
            {showWordmark ? (
                <>
                    <text
                        className="giglog-logo__wordmark"
                        x={textX}
                        y={showSubtitleText ? "30" : "31"}
                        fill="currentColor"
                        fontFamily="Poppins, sans-serif"
                        fontSize="24"
                        fontWeight="600"
                        letterSpacing="0.15"
                    >
                        GigLog
                    </text>
                    {showSubtitleText ? (
                        <text
                            x={textX}
                            y="41.5"
                            fill="currentColor"
                            fontFamily="Poppins, sans-serif"
                            fontSize="5.3"
                            fontWeight="500"
                            letterSpacing="0"
                        >
                            Track Your Gigs. Fuel Your Freedom.
                        </text>
                    ) : null}
                </>
            ) : null}
        </svg>
    );
};

export default GigLogLogoIcon;
