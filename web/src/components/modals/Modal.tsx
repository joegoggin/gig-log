import { ReactNode } from "react";

type ModalProps = {
    className?: string;
    showModal: boolean;
    children: ReactNode;
};

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
