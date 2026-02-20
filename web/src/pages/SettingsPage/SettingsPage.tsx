import { useState } from "react";
import styles from "./SettingsPage.module.scss";
import type { ThemeMode } from "@/lib/appearance";
import useForm from "@/hooks/useForm";
import useFormMutation from "@/hooks/useFormMutation";
import Button from "@/components/core/Button/Button";
import Form from "@/components/core/Form/Form";
import { NotificationType } from "@/components/core/Notification/Notification";
import TextInput from "@/components/core/TextInput/TextInput";
import { useAuth } from "@/contexts/AuthContext";
import { useAppearance } from "@/contexts/AppearanceContext";
import { useNotification } from "@/contexts/NotificationContext";
import api from "@/lib/axios";

type ChangePasswordFormData = {
    current_password: string;
    new_password: string;
    confirm: string;
};

type ChangePasswordResponse = {
    message: string;
};

type RequestEmailChangeFormData = {
    new_email: string;
};

type RequestEmailChangeResponse = {
    message: string;
};

type ConfirmEmailChangeFormData = {
    auth_code: string;
};

type ConfirmEmailChangeResponse = {
    message: string;
};

type RequestEmailChangeMutationResult = {
    message: string;
    normalizedEmail: string;
};

type ThemeModeOption = {
    /** Theme mode value applied when selected */
    value: ThemeMode;
    /** Visible label for the mode option */
    label: string;
    /** Supporting copy describing the mode behavior */
    description: string;
};

const themeModeOptions: Array<ThemeModeOption> = [
    {
        value: "system",
        label: "System",
        description: "Automatically matches your device preference.",
    },
    {
        value: "light",
        label: "Light",
        description: "Keeps surfaces bright for daytime readability.",
    },
    {
        value: "dark",
        label: "Dark",
        description: "Uses darker surfaces for lower-glare sessions.",
    },
];

/**
 * The authenticated account settings page for security and appearance workflows.
 * Lets signed-in users change their password, complete a verified email-change
 * flow, and choose a persisted theme mode preference.
 *
 * Route: `/settings`
 *
 * ## Props
 *
 * - None.
 *
 * ## Related Components
 *
 * - `Form` - Handles password and email settings submissions.
 * - `TextInput` - Captures password, email, and confirmation code fields.
 * - `Button` - Submits account-security actions.
 * - `MainLayout` - Wraps authenticated settings content.
 */
function SettingsPage() {
    const { user, refreshUser } = useAuth();
    const { mode, resolvedTheme, setMode } = useAppearance();
    const { addNotification } = useNotification();
    const [pendingEmailChange, setPendingEmailChange] = useState<string | null>(null);
    const {
        data: passwordData,
        errors: passwordErrors,
        setData: setPasswordData,
        setErrors: setPasswordErrors,
    } = useForm<ChangePasswordFormData>({
        current_password: "",
        new_password: "",
        confirm: "",
    });
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

    const changePasswordMutation = useFormMutation<
        ChangePasswordResponse,
        void
    >({
        mutationFn: async () => {
            const response = await api.post<ChangePasswordResponse>(
                "/auth/change-password",
                {
                    current_password: passwordData.current_password,
                    new_password: passwordData.new_password,
                    confirm: passwordData.confirm,
                },
            );
            return response.data;
        },
        onSuccess: (response) => {
            addNotification({
                type: NotificationType.SUCCESS,
                title: "Password Updated",
                message: response.message,
            });
            setPasswordData("current_password", "");
            setPasswordData("new_password", "");
            setPasswordData("confirm", "");
        },
        onError: setPasswordErrors,
        fallbackError: "Failed to change password",
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

    const handlePasswordSubmit = () => {
        changePasswordMutation.mutate();
    };

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
                <p className={styles["settings-page__eyebrow"]}>Account and appearance</p>
                <h1>Settings</h1>
                <p className={styles["settings-page__lead"]}>
                    Keep your credentials current and personalize how GigLog looks.
                </p>
                {user?.email && (
                    <p className={styles["settings-page__current-email"]}>
                        Signed in as <strong>{user.email}</strong>
                    </p>
                )}
            </header>

            <div className={styles["settings-page__grid"]}>
                <article className={styles["settings-page__panel"]}>
                    <h2>Change Password</h2>
                    <p className={styles["settings-page__panel-lead"]}>
                        Use your current password to set a new one for this account.
                    </p>
                    <Form onSubmit={handlePasswordSubmit}>
                        <TextInput
                            name="current_password"
                            placeholder="Current Password"
                            password
                            data={passwordData}
                            setData={setPasswordData}
                            errors={passwordErrors}
                        />
                        <TextInput
                            name="new_password"
                            placeholder="New Password"
                            password
                            data={passwordData}
                            setData={setPasswordData}
                            errors={passwordErrors}
                        />
                        <TextInput
                            name="confirm"
                            placeholder="Confirm New Password"
                            password
                            data={passwordData}
                            setData={setPasswordData}
                            errors={passwordErrors}
                        />
                        <Button type="submit">Change Password</Button>
                    </Form>
                </article>

                <article className={styles["settings-page__panel"]}>
                    <h2>Change Email</h2>
                    <p className={styles["settings-page__panel-lead"]}>
                        Verify ownership of your new email before replacing your current one.
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
            </div>

            <article
                className={`${styles["settings-page__panel"]} ${styles["settings-page__panel--appearance"]}`}
            >
                <h2>Theme Mode</h2>
                <p className={styles["settings-page__panel-lead"]}>
                    Choose how GigLog handles light and dark surfaces across the app.
                </p>

                <fieldset className={styles["settings-page__theme-fieldset"]}>
                    <legend className={styles["settings-page__theme-legend"]}>
                        Appearance mode
                    </legend>

                    <div className={styles["settings-page__theme-options"]}>
                        {themeModeOptions.map((option) => {
                            const inputId = `theme-mode-${option.value}`;
                            const isActive = mode === option.value;

                            return (
                                <label
                                    key={option.value}
                                    htmlFor={inputId}
                                    className={`${styles["settings-page__theme-option"]} ${
                                        isActive
                                            ? styles[
                                                  "settings-page__theme-option--active"
                                              ]
                                            : ""
                                    }`}
                                >
                                    <input
                                        id={inputId}
                                        type="radio"
                                        name="theme-mode"
                                        value={option.value}
                                        checked={isActive}
                                        onChange={() => setMode(option.value)}
                                    />
                                    <span
                                        className={
                                            styles["settings-page__theme-option-label"]
                                        }
                                    >
                                        {option.label}
                                    </span>
                                    <span
                                        className={
                                            styles["settings-page__theme-option-description"]
                                        }
                                    >
                                        {option.description}
                                    </span>
                                </label>
                            );
                        })}
                    </div>
                </fieldset>

                <p className={styles["settings-page__theme-status"]}>
                    Active theme: <strong>{resolvedTheme === "dark" ? "Dark" : "Light"}</strong>
                    {mode === "system" ? " (following your device setting)." : "."}
                </p>
            </article>
        </section>
    );
}

export default SettingsPage;
