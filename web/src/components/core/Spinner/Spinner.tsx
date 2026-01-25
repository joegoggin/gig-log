import styles from "./Spinner.module.scss";

/**
 * Props for the Spinner component.
 */
type SpinnerProps = {
    /** Additional CSS class names to apply to the spinner */
    className?: string;
    /** Label text displayed under the spinner */
    label?: string;
    /** Size of the spinner in pixels */
    size?: number;
};

/**
 *
 * A loading spinner with an optional label for progress feedback.
 *
 * ## Props
 *
 * - `className` - Additional CSS class names to apply to the spinner
 * - `label` - Label text displayed under the spinner (default: "Loading")
 * - `size` - Size of the spinner in pixels (default: 40)
 *
 * ## Example
 *
 * ```tsx
 * <Spinner label="Loading gigs" size={32} />
 * ```
 */
function Spinner({ className = "", label = "Loading", size = 40 }: SpinnerProps) {
    const spinnerClassName = className
        ? `${styles["spinner"]} ${className}`
        : styles["spinner"];

    return (
        <div className={spinnerClassName} role="status" aria-live="polite">
            <span
                className={styles["spinner__circle"]}
                style={{ width: size, height: size }}
            />
            {label && <span className={styles["spinner__label"]}>{label}</span>}
        </div>
    );
}

export default Spinner;
