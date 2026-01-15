import { type ReactNode } from "react";

/**
 * Props for the Modal component.
 */
type ModalProps = {
    /** Additional CSS class names to apply to the modal content */
    className?: string;
    /** Controls the visibility of the modal */
    showModal: boolean;
    /** Content to render inside the modal */
    children: ReactNode;
};

/**
 * A base modal component that provides an overlay container for modal content.
 * Renders children within a centered modal dialog when visible.
 *
 * Props:
 * - `className` - Additional CSS class names to apply to the modal content (default: "")
 * - `showModal` - Controls the visibility of the modal
 * - `children` - Content to render inside the modal
 *
 * @example
 * ```tsx
 * <Modal showModal={isOpen} className="my-modal">
 *   <p>Modal content here</p>
 * </Modal>
 * ```
 */
const Modal: React.FC<ModalProps> = ({
    className = "",
    showModal,
    children,
}) => {
    return (
        <>
            {showModal && (
                <div className="modal">
                    <div className={`modal__content ${className}`}>
                        {children}
                    </div>
                </div>
            )}
        </>
    );
};

export default Modal;
