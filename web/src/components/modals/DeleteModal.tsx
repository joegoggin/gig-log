import type { SetValue } from "types/SetValue";
import Button from "@/components/core/Button/Button";
import Modal from "@/components/modals/Modal";

/**
 * Props for the DeleteModal component.
 */
export type DeleteModalProps = {
    /** The ID of the item to be deleted */
    id: number;
    /** The display name of the item to be deleted */
    name: string;
    /** The table/entity type being deleted */
    table: string;
    /** List of related tables/entities that will also be affected */
    relatedTables: Array<string>;
    /** Controls the visibility of the modal */
    showModal: boolean;
    /** Function to update the modal visibility state */
    setShowModal: SetValue<boolean>;
};

/**
 * A confirmation modal for delete operations that warns users about cascading effects.
 * Displays the item name and lists all related data that will be permanently deleted.
 *
 * Props:
 * - `id` - The ID of the item to be deleted
 * - `name` - The display name of the item to be deleted
 * - `table` - The table/entity type being deleted
 * - `relatedTables` - List of related tables/entities that will also be affected
 * - `showModal` - Controls the visibility of the modal
 * - `setShowModal` - Function to update the modal visibility state
 *
 * @example
 * ```tsx
 * <DeleteModal
 *   id={123}
 *   name="Acme Corp"
 *   table="company"
 *   relatedTables={["jobs", "payments"]}
 *   showModal={showDeleteModal}
 *   setShowModal={setShowDeleteModal}
 * />
 * ```
 */
const DeleteModal: React.FC<DeleteModalProps> = ({
    id: _id,
    name,
    table,
    relatedTables,
    showModal,
    setShowModal,
}) => {
    const handleDelete = () => {
        // TODO: Implement actual delete API call using the id prop
        // Example: await api.delete(`/companies/${_id}`);
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
                    Deleting {name} will permanently delete all data associated
                    with this {table} including the following:
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
