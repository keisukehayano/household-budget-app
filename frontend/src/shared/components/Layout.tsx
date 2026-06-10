import { Link, Outlet } from "react-router";

export function Layout() {
    return (
        <div className="min-h-screen bg-slate-100">
            <header className="border-b bg-white px-6 py-4">
                <h1 className="text-xl font-bold text-slate-900">家計簿アプリ</h1>
            </header>

            <div className="flex">
                <aside className="min-h-[calc(100vh-65px)] w-64 border-r bg-white p-4">
                    <nav className="space-y-2">
                        <Link className="block rounded px-3 py-2 hover:bg-slate-100" to="/">
                        ダッシュボード
                        </Link>
                        <Link className="block rounded px-3 py-2 hover:bg-slate-100" to="/transactions">
                        取引一覧
                        </Link>
                        <Link className="block rounded px-3 py-2 hover:bg-slate-100" to="/transactions/new">
                        取引登録
                        </Link>
                        <Link className="block rounded px-3 py-2 hover:bg-slate-100" to="/categories">
                        カテゴリ管理
                        </Link>
                    </nav>
                </aside>

                <main className="flex-1 p-8">
                    <div className="rounded-xl bg-white p-6 shadow">
                        <Outlet />
                    </div>
                </main>
            </div>
        </div>
    );
}