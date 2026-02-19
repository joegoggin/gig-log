import React, { useEffect, useState } from "react";
import Button, { ButtonVariant } from "@/components/core/Button/Button";
import PlayIcon from "@/components/icons/PlayIcon";
import PauseIcon from "@/components/icons/PauseIcon";
import StopIcon from "@/components/icons/StopIcon";
import styles from "./WorkSessionTimer.module.scss";

export type TimerStatus = "idle" | "running" | "paused" | "completed";

export type WorkSessionTimerProps = {
    /** The current state of the work session */
    status: TimerStatus;
    /** The ISO timestamp or ms epoch when the current running segment started */
    startTime?: string | number | null;
    /** Time already accumulated in previous segments (in seconds) */
    accumulatedSeconds?: number;
    /** Callback fired when the user starts the timer from idle */
    onStart?: () => void;
    /** Callback fired when the user pauses an active timer */
    onPause?: () => void;
    /** Callback fired when the user resumes a paused timer */
    onResume?: () => void;
    /** Callback fired when the user completes/stops the timer */
    onComplete?: () => void;
};

/**
 * Format a total number of seconds into an HH:MM:SS string.
 */
function formatSeconds(totalSeconds: number): string {
    const s = Math.floor(totalSeconds % 60);
    const m = Math.floor((totalSeconds / 60) % 60);
    const h = Math.floor(totalSeconds / 3600);

    const pad = (num: number) => num.toString().padStart(2, "0");
    return `${pad(h)}:${pad(m)}:${pad(s)}`;
}

/**
 * A real-time ticking clock UI for tracking work sessions.
 * 
 * Takes in a status and the time a segment started, calculating the 
 * live elapsed time without parent re-renders while running.
 * 
 * ## Props
 * 
 * - `status` - The current status (idle, running, paused, completed)
 * - `startTime` - ISO timestamp when the current running segment started
 * - `accumulatedSeconds` - Base total seconds already worked prior to `startTime`
 * - `onStart` - Callback for start action
 * - `onPause` - Callback for pause action
 * - `onResume` - Callback for resume action
 * - `onComplete` - Callback for complete action
 * 
 * ## Example
 * 
 * ```tsx
 * <WorkSessionTimer 
 *   status="running" 
 *   startTime={new Date().toISOString()} 
 *   accumulatedSeconds={120}
 *   onPause={() => pauseSession()}
 *   onComplete={() => completeSession()}
 * />
 * ```
 */
const WorkSessionTimer: React.FC<WorkSessionTimerProps> = ({
    status,
    startTime = null,
    accumulatedSeconds = 0,
    onStart,
    onPause,
    onResume,
    onComplete,
}) => {
    const [displaySeconds, setDisplaySeconds] = useState(accumulatedSeconds);

    useEffect(() => {
        // Function to calculate exact elapsed time based on system clock
        const calculateElapsed = () => {
            let elapsed = accumulatedSeconds;
            if (status === "running" && startTime) {
                const startMs = typeof startTime === "string" ? new Date(startTime).getTime() : startTime;
                const nowMs = Date.now();
                // Avoid negative elapsed time if client clock is slightly behind server clock
                const diffMs = Math.max(0, nowMs - startMs);
                elapsed += Math.floor(diffMs / 1000);
            }
            return elapsed;
        };

        // Initialize right away
        setDisplaySeconds(calculateElapsed());

        // If not running, no need for an interval
        if (status !== "running") {
            return;
        }

        // Tick every half second to ensure we don't visibly "skip" seconds
        const intervalId = setInterval(() => {
            setDisplaySeconds(calculateElapsed());
        }, 500);

        return () => clearInterval(intervalId);
    }, [status, startTime, accumulatedSeconds]);

    return (
        <div className={styles.container}>
            <div className={styles.display}>
                <span className={styles.timeText}>{formatSeconds(displaySeconds)}</span>
                {status === "paused" && <span className={styles.pausedBadge}>PAUSED</span>}
                {status === "completed" && <span className={styles.completedBadge}>COMPLETED</span>}
            </div>
            
            {status !== "completed" && (
                <div className={styles.controls}>
                    {status === "idle" && (
                        <Button variant={ButtonVariant.PRIMARY} onClick={onStart} className={styles.controlButton}>
                            <PlayIcon /> Start
                        </Button>
                    )}
                    
                    {status === "running" && (
                        <>
                            <Button variant={ButtonVariant.SECONDARY} onClick={onPause} className={styles.controlButton}>
                                <PauseIcon /> Pause
                            </Button>
                            <Button variant={ButtonVariant.PRIMARY} onClick={onComplete} className={styles.controlButton}>
                                <StopIcon /> Complete
                            </Button>
                        </>
                    )}
                    
                    {status === "paused" && (
                        <>
                            <Button variant={ButtonVariant.SECONDARY} onClick={onResume} className={styles.controlButton}>
                                <PlayIcon /> Resume
                            </Button>
                            <Button variant={ButtonVariant.PRIMARY} onClick={onComplete} className={styles.controlButton}>
                                <StopIcon /> Complete
                            </Button>
                        </>
                    )}
                </div>
            )}
        </div>
    );
};

export default WorkSessionTimer;
