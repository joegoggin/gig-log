import { createFileRoute } from "@tanstack/react-router";
import axios from "axios";

type HelloResponse = {
    message: string;
};

export const Route = createFileRoute("/")({
    component: App,
    loader: async () => {
        try {
            const { data } = await axios.get<HelloResponse>(
                "http://localhost:8000/",
            );

            return data;
        } catch (error) {
            return { message: "failed to fetch from api" };
        }
    },
});

function App() {
    const data = Route.useLoaderData();
    return (
        <div>
            <h1>GigLog</h1>
            <h2>{data?.message}</h2>
        </div>
    );
}
