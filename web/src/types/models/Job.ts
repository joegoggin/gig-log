export type JobPaymentType = "hourly" | "payouts";

export type Job = {
    id: string;
    company_id: string;
    user_id: string;
    title: string;
    payment_type: JobPaymentType;
    number_of_payouts: number | null;
    payout_amount: string | null;
    hourly_rate: string | null;
    created_at: string;
    updated_at: string;
};

export type JobsListResponse = {
    jobs: Array<Job>;
};
