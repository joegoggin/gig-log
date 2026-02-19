import React from "react";

/**
 * A right-pointing triangle representing a "play" or "start" action.
 */
const PlayIcon: React.FC = () => {
    return (
        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            width="24"
            height="24"
            fill="currentColor"
        >
            <path d="M8 5v14l11-7z" />
        </svg>
    );
};

export default PlayIcon;
