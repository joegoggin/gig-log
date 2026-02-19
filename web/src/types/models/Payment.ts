export type PaymentPayoutType =
    | "paypal"
    | "cash"
    | "check"
    | "zelle"
    | "venmo"
    | "direct_deposit";

export type Payment = {
    id: string;
    user_id: string;
    company_id: string;
    total: string;
    payout_type: PaymentPayoutType;
    expected_payout_date: string | null;
    expected_transfer_date: string | null;
    transfer_initiated: boolean;
    payment_received: boolean;
    transfer_received: boolean;
    tax_withholdings_covered: boolean;
    created_at: string;
    updated_at: string;
};

export type PaymentsListResponse = {
    payments: Array<Payment>;
};

export type PaymentResponse = {
    payment: Payment;
};

export type DeletePaymentResponse = {
    message: string;
};
