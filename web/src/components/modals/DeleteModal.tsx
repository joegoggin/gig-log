import Button from "@/components/core/Button/Button";
import Modal from "@/components/modals/Modal";
import type { SetValue } from "types/SetValue";

export type DeleteModalProps = {
    id: number;
    name: string;
    table: string;
    relatedTables: string[];
    showModal: boolean;
    setShowModal: SetValue<boolean>;
};

const DeleteModal: React.FC<DeleteModalProps> = ({
    id,
    name,
    table,
    relatedTables,
    showModal,
    setShowModal,
}) => {
    const handleDelete = () => {
        // router.delete(`/companies/${id}?prompt_user=false`);
        setShowModal(false);
    };

    const handleClose = () => {
        setShowModal(false);
    };

    return (
        <Modal className="delete-modal" showModal={showModal}>
            <div className="delete-modal__content">
                <h3>Are you sure you want to delete {name}?</h3>
                <h5>
                    Deleting {name} will permanently all data associated with
                    this {table} including the following:
                </h5>
                <ul>
                    {relatedTables.map((relatedTable) => (
                        <li key={relatedTable}>{relatedTable}</li>
                    ))}
                </ul>
                <div className="delete-modal__buttons">
                    <Button
                        className="delete-modal__delete"
                        onClick={handleDelete}
                    >
                        Delete
                    </Button>
                    <Button onClick={handleClose}>Cancel</Button>
                </div>
            </div>
        </Modal>
    );
};

export default DeleteModal;
