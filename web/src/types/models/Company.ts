export type Company = {
    id: string;
    user_id: string;
    name: string;
    requires_tax_withholdings: boolean;
    tax_withholding_rate: string | null;
    created_at: string;
    updated_at: string;
};

export type CompanyDetails = Company & {
    payment_total: string | number;
    hours: string;
};

export type CompanyJob = {
    id: string;
    title: string;
};

export type CompanyPayment = {
    id: string;
    total: string | number;
    payout_type: string;
    payment_received: boolean;
    transfer_received: boolean;
};

export type CompanyDetailResponse = {
    company: CompanyDetails;
    paginated_jobs: Array<CompanyJob>;
    jobs_has_more: boolean;
    paginated_payments: Array<CompanyPayment>;
    payments_has_more: boolean;
};
