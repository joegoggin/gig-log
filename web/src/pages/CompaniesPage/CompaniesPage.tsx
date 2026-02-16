import { useEffect, useState } from "react";
import { useNavigate } from "@tanstack/react-router";
import styles from "./CompaniesPage.module.scss";
import type { Company } from "@/types/models/Company";
import AddIcon from "@/components/icons/AddIcon";
import DeleteIcon from "@/components/icons/DeleteIcon";
import EditIcon from "@/components/icons/EditIcon";
import InfoIcon from "@/components/icons/InfoIcon";
import { NotificationType } from "@/components/core/Notification/Notification";
import { useNotification } from "@/contexts/NotificationContext";
import api from "@/lib/axios";

type CompaniesListResponse = {
    companies: Array<Company>;
};

type CompaniesPageProps = {
    /** Optional preloaded companies for deterministic rendering in stories/tests */
    initialCompanies?: Array<Company>;
};

/**
 * The authenticated companies index page.
 * Displays all companies owned by the current user and provides navigation
 * to create, view, and edit flows.
 *
 * Route: `/companies`
 *
 * ## Props
 *
 * - `initialCompanies` - Optional preloaded company records used by stories/tests.
 *
 * ## Related Components
 *
 * - `Button` - Navigates to create, detail, and edit routes.
 */
function CompaniesPage({ initialCompanies }: CompaniesPageProps) {
    const navigate = useNavigate();
    const { addNotification } = useNotification();
    const hasInitialCompanies = initialCompanies !== undefined;
    const [companies, setCompanies] = useState<Array<Company>>(initialCompanies || []);
    const [isLoading, setIsLoading] = useState<boolean>(!hasInitialCompanies);
    const [deletingCompanyId, setDeletingCompanyId] = useState<string | null>(null);

    useEffect(() => {
        if (hasInitialCompanies) {
            return;
        }

        const fetchCompanies = async () => {
            try {
                const response = await api.get<CompaniesListResponse>("/companies");
                setCompanies(response.data.companies);
            } catch {
                addNotification({
                    type: NotificationType.ERROR,
                    title: "Companies Unavailable",
                    message: "Failed to load companies.",
                });
            } finally {
                setIsLoading(false);
            }
        };

        fetchCompanies();
    }, [addNotification, hasInitialCompanies]);

    const handleDeleteCompany = async (company: Company) => {
        const shouldDelete = window.confirm(
            `Delete "${company.name}"? This also removes related jobs and payments.`,
        );

        if (!shouldDelete || deletingCompanyId) {
            return;
        }

        setDeletingCompanyId(company.id);

        try {
            await api.delete(`/companies/${company.id}`);
            setCompanies((currentCompanies) =>
                currentCompanies.filter((currentCompany) => currentCompany.id !== company.id),
            );
            addNotification({
                type: NotificationType.SUCCESS,
                title: "Company Deleted",
                message: `${company.name} was deleted successfully.`,
            });
        } catch {
            addNotification({
                type: NotificationType.ERROR,
                title: "Delete Failed",
                message: "Unable to delete this company right now.",
            });
        } finally {
            setDeletingCompanyId(null);
        }
    };

    const navigateTo = (to: string) => {
        navigate({ to });
    };

    return (
        <section className={styles["companies-page"]}>
            <header className={styles["companies-page__header"]}>
                <div>
                    <p className={styles["companies-page__eyebrow"]}>Client records</p>
                    <h1>Companies</h1>
                    <p className={styles["companies-page__lead"]}>
                        Manage the clients you track work and payments for.
                    </p>
                </div>
                <button
                    className={`${styles["companies-page__icon-button"]} ${styles["companies-page__add-action"]}`}
                    onClick={() => {
                        navigateTo("/companies/create");
                    }}
                    type="button"
                >
                    <AddIcon />
                    <p>Add Company</p>
                </button>
            </header>

            {isLoading && <p>Loading companies...</p>}

            {!isLoading && companies.length === 0 && (
                <div className={styles["companies-page__empty"]}>
                    <p>No companies yet. Create your first company to get started.</p>
                </div>
            )}

            {!isLoading && companies.length > 0 && (
                <div className={styles["companies-page__grid"]}>
                    {companies.map((company) => (
                        <article
                            key={company.id}
                            className={styles["companies-page__company-card"]}
                        >
                            <h3>{company.name}</h3>
                            <p>
                                Tax withholdings:{" "}
                                {company.requires_tax_withholdings
                                    ? `Enabled${company.tax_withholding_rate ? ` (${company.tax_withholding_rate}%)` : ""}`
                                    : "Disabled"}
                            </p>
                            <div className={styles["companies-page__actions"]}>
                                <button
                                    className={`${styles["companies-page__icon-button"]} ${styles["companies-page__add-action"]}`}
                                    onClick={() => {
                                        navigateTo("/jobs");
                                    }}
                                    type="button"
                                >
                                    <AddIcon />
                                    <p>Add Job</p>
                                </button>
                                <button
                                    className={`${styles["companies-page__icon-button"]} ${styles["companies-page__add-action"]}`}
                                    onClick={() => {
                                        navigateTo("/payments");
                                    }}
                                    type="button"
                                >
                                    <AddIcon />
                                    <p>Add Payment</p>
                                </button>
                                <button
                                    className={`${styles["companies-page__icon-button"]} ${styles["companies-page__view-action"]}`}
                                    onClick={() => {
                                        navigateTo(`/companies/${company.id}`);
                                    }}
                                    type="button"
                                >
                                    <InfoIcon />
                                    <p>View Company</p>
                                </button>
                                <button
                                    className={`${styles["companies-page__icon-button"]} ${styles["companies-page__edit-action"]}`}
                                    onClick={() => {
                                        navigateTo(`/companies/${company.id}/edit`);
                                    }}
                                    type="button"
                                >
                                    <EditIcon />
                                    <p>Edit Company</p>
                                </button>
                                <button
                                    className={`${styles["companies-page__icon-button"]} ${styles["companies-page__delete-action"]}`}
                                    onClick={() => {
                                        void handleDeleteCompany(company);
                                    }}
                                    type="button"
                                >
                                    <DeleteIcon />
                                    <p>
                                        {deletingCompanyId === company.id
                                            ? "Deleting Company..."
                                            : "Delete Company"}
                                    </p>
                                </button>
                            </div>
                        </article>
                    ))}
                </div>
            )}
        </section>
    );
}

export default CompaniesPage;
