import type { Meta, StoryObj } from "@storybook/react-vite";
import AddIcon from "@/components/icons/AddIcon";
import BackIcon from "@/components/icons/BackIcon";
import CheckIcon from "@/components/icons/CheckIcon";
import CloseIcon from "@/components/icons/CloseIcon";
import CompanyIcon from "@/components/icons/CompanyIcon";
import DeleteIcon from "@/components/icons/DeleteIcon";
import EditIcon from "@/components/icons/EditIcon";
import ErrorIcon from "@/components/icons/ErrorIcon";
import GigLogLogoIcon from "@/components/icons/GigLogLogoIcon";
import HamburgerIcon from "@/components/icons/HamburgerIcon";
import HomeIcon from "@/components/icons/HomeIcon";
import InfoIcon from "@/components/icons/InfoIcon";
import JobsIcon from "@/components/icons/JobsIcon";
import LogOutIcon from "@/components/icons/LogOutIcon";
import PaymentIcon from "@/components/icons/PaymentIcon";
import SettingsIcon from "@/components/icons/SettingsIcon";
import WarningIcon from "@/components/icons/WarningIcon";

const IconGallery = () => {
    const icons = [
        { name: "AddIcon", component: AddIcon },
        { name: "BackIcon", component: BackIcon },
        { name: "CheckIcon", component: CheckIcon },
        { name: "CloseIcon", component: CloseIcon },
        { name: "CompanyIcon", component: CompanyIcon },
        { name: "DeleteIcon", component: DeleteIcon },
        { name: "EditIcon", component: EditIcon },
        { name: "ErrorIcon", component: ErrorIcon },
        { name: "GigLogLogoIcon", component: GigLogLogoIcon },
        { name: "HamburgerIcon", component: HamburgerIcon },
        { name: "HomeIcon", component: HomeIcon },
        { name: "InfoIcon", component: InfoIcon },
        { name: "JobsIcon", component: JobsIcon },
        { name: "LogOutIcon", component: LogOutIcon },
        { name: "PaymentIcon", component: PaymentIcon },
        { name: "SettingsIcon", component: SettingsIcon },
        { name: "WarningIcon", component: WarningIcon },
    ];

    return (
        <div
            style={{
                display: "grid",
                gridTemplateColumns: "repeat(auto-fill, minmax(120px, 1fr))",
                gap: "24px",
                padding: "16px",
            }}
        >
            {icons.map(({ name, component: Icon }) => (
                <div
                    key={name}
                    style={{
                        display: "flex",
                        flexDirection: "column",
                        alignItems: "center",
                        gap: "8px",
                        padding: "16px",
                        border: "1px solid var(--text-color)",
                        borderRadius: "8px",
                        color: "var(--text-color)",
                    }}
                >
                    <Icon />
                    <span style={{ fontSize: "12px" }}>
                        {name}
                    </span>
                </div>
            ))}
        </div>
    );
};

const meta: Meta<typeof IconGallery> = {
    title: "Icons/All Icons",
    component: IconGallery,
    tags: ["autodocs"],
};

export default meta;
type Story = StoryObj<typeof IconGallery>;

export const Gallery: Story = {};

// Wrapper for individual icon stories to inherit theme color
const IconWrapper = ({ children }: { children: React.ReactNode }) => (
    <div style={{ color: "var(--text-color)" }}>{children}</div>
);

// Individual icon stories
export const Add: StoryObj<typeof AddIcon> = {
    render: () => (
        <IconWrapper>
            <AddIcon />
        </IconWrapper>
    ),
};

export const Back: StoryObj<typeof BackIcon> = {
    render: () => (
        <IconWrapper>
            <BackIcon />
        </IconWrapper>
    ),
};

export const Check: StoryObj<typeof CheckIcon> = {
    render: () => (
        <IconWrapper>
            <CheckIcon />
        </IconWrapper>
    ),
};

export const Close: StoryObj<typeof CloseIcon> = {
    render: () => (
        <IconWrapper>
            <CloseIcon />
        </IconWrapper>
    ),
};

export const Company: StoryObj<typeof CompanyIcon> = {
    render: () => (
        <IconWrapper>
            <CompanyIcon />
        </IconWrapper>
    ),
};

export const Delete: StoryObj<typeof DeleteIcon> = {
    render: () => (
        <IconWrapper>
            <DeleteIcon />
        </IconWrapper>
    ),
};

export const Edit: StoryObj<typeof EditIcon> = {
    render: () => (
        <IconWrapper>
            <EditIcon />
        </IconWrapper>
    ),
};

export const Error: StoryObj<typeof ErrorIcon> = {
    render: () => (
        <IconWrapper>
            <ErrorIcon />
        </IconWrapper>
    ),
};

export const GigLogLogo: StoryObj<typeof GigLogLogoIcon> = {
    render: () => (
        <IconWrapper>
            <GigLogLogoIcon />
        </IconWrapper>
    ),
};

export const Hamburger: StoryObj<typeof HamburgerIcon> = {
    render: () => (
        <IconWrapper>
            <HamburgerIcon />
        </IconWrapper>
    ),
};

export const Home: StoryObj<typeof HomeIcon> = {
    render: () => (
        <IconWrapper>
            <HomeIcon />
        </IconWrapper>
    ),
};

export const Info: StoryObj<typeof InfoIcon> = {
    render: () => (
        <IconWrapper>
            <InfoIcon />
        </IconWrapper>
    ),
};

export const Jobs: StoryObj<typeof JobsIcon> = {
    render: () => (
        <IconWrapper>
            <JobsIcon />
        </IconWrapper>
    ),
};

export const LogOut: StoryObj<typeof LogOutIcon> = {
    render: () => (
        <IconWrapper>
            <LogOutIcon />
        </IconWrapper>
    ),
};

export const Payment: StoryObj<typeof PaymentIcon> = {
    render: () => (
        <IconWrapper>
            <PaymentIcon />
        </IconWrapper>
    ),
};

export const Settings: StoryObj<typeof SettingsIcon> = {
    render: () => (
        <IconWrapper>
            <SettingsIcon />
        </IconWrapper>
    ),
};

export const Warning: StoryObj<typeof WarningIcon> = {
    render: () => (
        <IconWrapper>
            <WarningIcon />
        </IconWrapper>
    ),
};
