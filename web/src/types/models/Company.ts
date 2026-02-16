export type Company = {
    id: string;
    user_id: string;
    name: string;
    requires_tax_withholdings: boolean;
    tax_withholding_rate: string | null;
    created_at: string;
    updated_at: string;
};
