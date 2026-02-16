import { useEffect, useState } from "react";
import styles from "./CompaniesPage.module.scss";
import type { Company } from "@/types/models/Company";
import Button, { ButtonVariant } from "@/components/core/Button/Button";
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
    const { addNotification } = useNotification();
    const hasInitialCompanies = initialCompanies !== undefined;
    const [companies, setCompanies] = useState<Array<Company>>(initialCompanies || []);
    const [isLoading, setIsLoading] = useState<boolean>(!hasInitialCompanies);

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

    return (
        <section className={styles["companies-page"]}>
            <header className={styles["companies-page__header"]}>
                <div>
                    <h1>Companies</h1>
                    <p>Manage the clients you track work and payments for.</p>
                </div>
                <Button href="/companies/create">Create Company</Button>
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
                            <h2>{company.name}</h2>
                            <p>
                                Tax withholdings:{" "}
                                {company.requires_tax_withholdings
                                    ? `Enabled${company.tax_withholding_rate ? ` (${company.tax_withholding_rate}%)` : ""}`
                                    : "Disabled"}
                            </p>
                            <div className={styles["companies-page__actions"]}>
                                <Button href={`/companies/${company.id}`}>View Company</Button>
                                <Button
                                    href={`/companies/${company.id}/edit`}
                                    variant={ButtonVariant.SECONDARY}
                                >
                                    Edit Company
                                </Button>
                            </div>
                        </article>
                    ))}
                </div>
            )}
        </section>
    );
}

export default CompaniesPage;
