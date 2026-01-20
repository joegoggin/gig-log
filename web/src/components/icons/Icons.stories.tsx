import type { Meta, StoryObj } from "@storybook/react-vite";
import AddIcon from "./AddIcon";
import BackIcon from "./BackIcon";
import CheckIcon from "./CheckIcon";
import CloseIcon from "./CloseIcon";
import CompanyIcon from "./CompanyIcon";
import DeleteIcon from "./DeleteIcon";
import EditIcon from "./EditIcon";
import ErrorIcon from "./ErrorIcon";
import HamburgerIcon from "./HamburgerIcon";
import HomeIcon from "./HomeIcon";
import InfoIcon from "./InfoIcon";
import JobsIcon from "./JobsIcon";
import LogOutIcon from "./LogOutIcon";
import PaymentIcon from "./PaymentIcon";
import SettingsIcon from "./SettingsIcon";
import WarningIcon from "./WarningIcon";

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
                        border: "1px solid #e0e0e0",
                        borderRadius: "8px",
                    }}
                >
                    <Icon />
                    <span style={{ fontSize: "12px", color: "#666" }}>
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

// Individual icon stories
export const Add: StoryObj<typeof AddIcon> = {
    render: () => <AddIcon />,
};

export const Back: StoryObj<typeof BackIcon> = {
    render: () => <BackIcon />,
};

export const Check: StoryObj<typeof CheckIcon> = {
    render: () => <CheckIcon />,
};

export const Close: StoryObj<typeof CloseIcon> = {
    render: () => <CloseIcon />,
};

export const Company: StoryObj<typeof CompanyIcon> = {
    render: () => <CompanyIcon />,
};

export const Delete: StoryObj<typeof DeleteIcon> = {
    render: () => <DeleteIcon />,
};

export const Edit: StoryObj<typeof EditIcon> = {
    render: () => <EditIcon />,
};

export const Error: StoryObj<typeof ErrorIcon> = {
    render: () => <ErrorIcon />,
};

export const Hamburger: StoryObj<typeof HamburgerIcon> = {
    render: () => <HamburgerIcon />,
};

export const Home: StoryObj<typeof HomeIcon> = {
    render: () => <HomeIcon />,
};

export const Info: StoryObj<typeof InfoIcon> = {
    render: () => <InfoIcon />,
};

export const Jobs: StoryObj<typeof JobsIcon> = {
    render: () => <JobsIcon />,
};

export const LogOut: StoryObj<typeof LogOutIcon> = {
    render: () => <LogOutIcon />,
};

export const Payment: StoryObj<typeof PaymentIcon> = {
    render: () => <PaymentIcon />,
};

export const Settings: StoryObj<typeof SettingsIcon> = {
    render: () => <SettingsIcon />,
};

export const Warning: StoryObj<typeof WarningIcon> = {
    render: () => <WarningIcon />,
};
