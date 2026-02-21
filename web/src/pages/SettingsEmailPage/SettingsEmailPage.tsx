import { useLocation } from "@tanstack/react-router";
import { useState } from "react";
import styles from "@/pages/SettingsPage/SettingsPage.module.scss";
import useForm from "@/hooks/useForm";
import useFormMutation from "@/hooks/useFormMutation";
import Button, { ButtonVariant } from "@/components/core/Button/Button";
import Form from "@/components/core/Form/Form";
import { NotificationType } from "@/components/core/Notification/Notification";
import TextInput from "@/components/core/TextInput/TextInput";
import { useAuth } from "@/contexts/AuthContext";
import { useNotification } from "@/contexts/NotificationContext";
import api from "@/lib/axios";
import { settingsSections } from "@/pages/SettingsPage/settingsSections";

type RequestEmailChangeFormData = {
    new_email: string;
};

type ConfirmEmailChangeFormData = {
    auth_code: string;
};

type RequestEmailChangeResponse = {
    message: string;
};

type ConfirmEmailChangeResponse = {
    message: string;
};

type RequestEmailChangeMutationResult = {
    message: string;
    normalizedEmail: string;
};

/**
 * The authenticated email settings page.
 * Guides users through a two-step, code-verified email change workflow.
 *
 * Route: `/settings/email`
 *
 * ## Props
 *
 * - None.
 *
 * ## Related Components
 *
 * - `Form` - Handles email change request/confirm submissions.
 * - `TextInput` - Captures new email and confirmation code fields.
 * - `Button` - Triggers email change actions.
 */
function SettingsEmailPage() {
    const { pathname } = useLocation();
    const { user, refreshUser } = useAuth();
    const { addNotification } = useNotification();
    const [pendingEmailChange, setPendingEmailChange] = useState<string | null>(null);
    const {
        data: requestEmailData,
        errors: requestEmailErrors,
        setData: setRequestEmailData,
        setErrors: setRequestEmailErrors,
    } = useForm<RequestEmailChangeFormData>({
        new_email: "",
    });
    const {
        data: confirmEmailData,
        errors: confirmEmailErrors,
        setData: setConfirmEmailData,
        setErrors: setConfirmEmailErrors,
    } = useForm<ConfirmEmailChangeFormData>({
        auth_code: "",
    });

    const requestEmailChangeMutation = useFormMutation<
        RequestEmailChangeMutationResult,
        void
    >({
        mutationFn: async () => {
            const normalizedEmail = requestEmailData.new_email.trim().toLowerCase();

            const response = await api.post<RequestEmailChangeResponse>(
                "/auth/request-email-change",
                {
                    new_email: normalizedEmail,
                },
            );

            return {
                message: response.data.message,
                normalizedEmail,
            };
        },
        onSuccess: ({ message, normalizedEmail }) => {
            setPendingEmailChange(normalizedEmail);
            setRequestEmailData("new_email", normalizedEmail);
            setConfirmEmailData("auth_code", "");

            addNotification({
                type: NotificationType.INFO,
                title: "Confirmation Code Sent",
                message,
            });
        },
        onError: setRequestEmailErrors,
        fallbackError: "Failed to request email change",
    });

    const confirmEmailChangeMutation = useFormMutation<
        ConfirmEmailChangeResponse,
        void
    >({
        mutationFn: async () => {
            const emailToConfirm =
                pendingEmailChange ?? requestEmailData.new_email.trim().toLowerCase();

            const response = await api.post<ConfirmEmailChangeResponse>(
                "/auth/confirm-email-change",
                {
                    new_email: emailToConfirm,
                    auth_code: confirmEmailData.auth_code,
                },
            );
            return response.data;
        },
        onSuccess: async (response) => {
            await refreshUser();
            addNotification({
                type: NotificationType.SUCCESS,
                title: "Email Updated",
                message: response.message,
            });

            setPendingEmailChange(null);
            setRequestEmailData("new_email", "");
            setConfirmEmailData("auth_code", "");
        },
        onError: setConfirmEmailErrors,
        fallbackError: "Failed to confirm email change",
    });

    const handleRequestEmailSubmit = () => {
        requestEmailChangeMutation.mutate();
    };

    const handleConfirmEmailSubmit = () => {
        if (!pendingEmailChange) {
            setRequestEmailErrors({
                new_email: "Request a confirmation code first.",
            });
            return;
        }

        confirmEmailChangeMutation.mutate();
    };

    return (
        <section className={styles["settings-page"]}>
            <header className={styles["settings-page__hero"]}>
                <p className={styles["settings-page__eyebrow"]}>Account security</p>
                <h1>Email settings</h1>
                <p className={styles["settings-page__lead"]}>
                    Verify ownership of your new email before replacing your current one.
                </p>
                {user?.email && (
                    <p className={styles["settings-page__current-email"]}>
                        Signed in as <strong>{user.email}</strong>
                    </p>
                )}
            </header>

            <nav
                className={styles["settings-page__subnav"]}
                aria-label="Settings sections"
            >
                {settingsSections.map((section) => {
                    const isActive =
                        pathname === section.href || pathname === `${section.href}/`;

                    return (
                        <Button
                            key={section.href}
                            href={section.href}
                            variant={
                                isActive
                                    ? ButtonVariant.PRIMARY
                                    : ButtonVariant.SECONDARY
                            }
                            className={styles["settings-page__subnav-button"]}
                        >
                            {section.label}
                        </Button>
                    );
                })}
            </nav>

            <article className={styles["settings-page__panel"]}>
                <h2>Change Email</h2>
                <p className={styles["settings-page__panel-lead"]}>
                    Request a confirmation code, then verify it to finish the update.
                </p>

                <div className={styles["settings-page__email-flow"]}>
                    <div className={styles["settings-page__email-step"]}>
                        <h3>1. Request confirmation code</h3>
                        <Form onSubmit={handleRequestEmailSubmit}>
                            <TextInput
                                name="new_email"
                                placeholder="New Email"
                                type="email"
                                autoCapitalize="none"
                                autoCorrect="off"
                                spellCheck={false}
                                data={requestEmailData}
                                setData={setRequestEmailData}
                                errors={requestEmailErrors}
                            />
                            <Button type="submit">
                                {pendingEmailChange
                                    ? "Resend Confirmation Code"
                                    : "Send Confirmation Code"}
                            </Button>
                        </Form>
                    </div>

                    <div className={styles["settings-page__email-step"]}>
                        <h3>2. Confirm new email</h3>
                        <p className={styles["settings-page__step-note"]}>
                            {pendingEmailChange
                                ? `Enter the code sent to ${pendingEmailChange}.`
                                : "Request a confirmation code first."}
                        </p>
                        <Form onSubmit={handleConfirmEmailSubmit}>
                            <TextInput
                                name="auth_code"
                                placeholder="Email Change Code"
                                data={confirmEmailData}
                                setData={setConfirmEmailData}
                                errors={confirmEmailErrors}
                            />
                            <Button type="submit">Confirm Email Change</Button>
                        </Form>
                    </div>
                </div>
            </article>
        </section>
    );
}

export default SettingsEmailPage;
