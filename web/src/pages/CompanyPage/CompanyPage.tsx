import { useEffect, useRef } from "react";
import { useQuery } from "@tanstack/react-query";
import styles from "./CompanyPage.module.scss";
import type { Company } from "@/types/models/Company";
import Button, { ButtonVariant } from "@/components/core/Button/Button";
import { NotificationType } from "@/components/core/Notification/Notification";
import { useNotification } from "@/contexts/NotificationContext";
import api from "@/lib/axios";

type CompanyPageProps = {
    /** Identifier of the company to display */
    companyId: string;
    /** Optional preloaded company for deterministic rendering in stories/tests */
    initialCompany?: Company | null;
};

type CompanyResponse = {
    company: Company;
};

/**
 * The authenticated company detail page.
 * Displays details for a single company selected by ID.
 *
 * Route: `/companies/$companyId`
 *
 * ## Props
 *
 * - `companyId` - Identifier of the company to fetch and render.
 * - `initialCompany` - Optional preloaded company used by stories/tests.
 *
 * ## Related Components
 *
 * - `Button` - Navigates back to the list and into edit flow.
 */
function CompanyPage({ companyId, initialCompany }: CompanyPageProps) {
    const { addNotification } = useNotification();
    const hasInitialCompany = initialCompany !== undefined;
    const hasShownErrorRef = useRef<boolean>(false);
    const {
        data: fetchedCompany,
        isLoading,
        isError,
    } = useQuery({
        queryKey: ["company", companyId],
        queryFn: async () => {
            const response = await api.get<CompanyResponse>(`/companies/${companyId}`);
            return response.data.company;
        },
        enabled: !hasInitialCompany,
        staleTime: Number.POSITIVE_INFINITY,
        refetchOnWindowFocus: false,
        retry: false,
    });

    useEffect(() => {
        hasShownErrorRef.current = false;
    }, [companyId]);

    useEffect(() => {
        if (isError && !hasShownErrorRef.current) {
            hasShownErrorRef.current = true;
            addNotification({
                type: NotificationType.ERROR,
                title: "Company Not Found",
                message: "Unable to load the requested company.",
            });
        }
    }, [addNotification, isError]);

    const company = hasInitialCompany ? initialCompany || null : fetchedCompany || null;

    return (
        <section className={styles["company-page"]}>
            <header className={styles["company-page__header"]}>
                <h1>Company</h1>
                <div className={styles["company-page__header-actions"]}>
                    <Button href="/companies" variant={ButtonVariant.SECONDARY}>
                        Back to Companies
                    </Button>
                    <Button href={`/companies/${companyId}/edit`}>Edit Company</Button>
                </div>
            </header>

            {isLoading && <p>Loading company...</p>}

            {!isLoading && !company && <p>This company could not be found.</p>}

            {!isLoading && company && (
                <article className={styles["company-page__details"]}>
                    <h2>{company.name}</h2>
                    <p>
                        Tax withholdings:{" "}
                        {company.requires_tax_withholdings
                            ? `Enabled${company.tax_withholding_rate ? ` (${company.tax_withholding_rate}%)` : ""}`
                            : "Disabled"}
                    </p>
                    <p>Created: {new Date(company.created_at).toLocaleString()}</p>
                    <p>Last updated: {new Date(company.updated_at).toLocaleString()}</p>
                </article>
            )}
        </section>
    );
}

export default CompanyPage;
