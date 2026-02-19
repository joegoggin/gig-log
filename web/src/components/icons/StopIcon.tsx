import React from "react";

/**
 * A square representing a "stop" or "complete" action.
 */
const StopIcon: React.FC = () => {
    return (
        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            width="24"
            height="24"
            fill="currentColor"
        >
            <path d="M6 6h12v12H6z" />
        </svg>
    );
};

export default StopIcon;
