import { createBrowserRouter } from "react-router";
import { Layout } from "../shared/components/Layout";
import { DashboardPage } from "../features/dashboard/pages/DashboardPage";
import { TransactionListPage } from "../features/dashboard/pages/TransactionListPage";
import { TransactionCreatePage } from "../features/dashboard/pages/TransactionCreatePage";
import { CategoryListPage } from "../features/dashboard/pages/CategoryListPage";

export const router = createBrowserRouter([
    {
        path: "/",
        Component: Layout,
        children: [
            {
                index: true,
                Component: DashboardPage,
            },
            {
                path: "transactions",
                Component: TransactionListPage,
            },
            {
                path: "transactions/new",
                Component: TransactionCreatePage,
            },
            {
                path: "categories",
                Component: CategoryListPage
            },
        ],
    },
]);