import { useLocation } from "@tanstack/react-router";
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

type ChangePasswordFormData = {
    current_password: string;
    new_password: string;
    confirm: string;
};

type ChangePasswordResponse = {
    message: string;
};

/**
 * The authenticated password settings page.
 * Allows users to update account credentials with current-password verification.
 *
 * Route: `/settings/password`
 *
 * ## Props
 *
 * - None.
 *
 * ## Related Components
 *
 * - `Form` - Handles password update submission.
 * - `TextInput` - Captures current/new password fields.
 * - `Button` - Submits the password update request.
 */
function SettingsPasswordPage() {
    const { pathname } = useLocation();
    const { user } = useAuth();
    const { addNotification } = useNotification();
    const { data, errors, setData, setErrors } = useForm<ChangePasswordFormData>({
        current_password: "",
        new_password: "",
        confirm: "",
    });

    const changePasswordMutation = useFormMutation<ChangePasswordResponse, void>({
        mutationFn: async () => {
            const response = await api.post<ChangePasswordResponse>(
                "/auth/change-password",
                {
                    current_password: data.current_password,
                    new_password: data.new_password,
                    confirm: data.confirm,
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
            setData("current_password", "");
            setData("new_password", "");
            setData("confirm", "");
        },
        onError: setErrors,
        fallbackError: "Failed to change password",
    });

    const handleSubmit = () => {
        changePasswordMutation.mutate();
    };

    return (
        <section className={styles["settings-page"]}>
            <header className={styles["settings-page__hero"]}>
                <p className={styles["settings-page__eyebrow"]}>Account security</p>
                <h1>Password settings</h1>
                <p className={styles["settings-page__lead"]}>
                    Keep your account secure by rotating credentials when needed.
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
                <h2>Change Password</h2>
                <p className={styles["settings-page__panel-lead"]}>
                    Use your current password to set a new one for this account.
                </p>
                <Form onSubmit={handleSubmit}>
                    <TextInput
                        name="current_password"
                        placeholder="Current Password"
                        password
                        data={data}
                        setData={setData}
                        errors={errors}
                    />
                    <TextInput
                        name="new_password"
                        placeholder="New Password"
                        password
                        data={data}
                        setData={setData}
                        errors={errors}
                    />
                    <TextInput
                        name="confirm"
                        placeholder="Confirm New Password"
                        password
                        data={data}
                        setData={setData}
                        errors={errors}
                    />
                    <Button type="submit">Change Password</Button>
                </Form>
            </article>
        </section>
    );
}

export default SettingsPasswordPage;
