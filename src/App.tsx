import { Outlet } from "react-router-dom";
import { Sidebar } from "./components/layout/Sidebar";

export default function App() {
  return (
    <div className="flex h-screen overflow-hidden">
      <Sidebar />
      <main className="flex flex-1 flex-col overflow-auto">
        <Outlet />
      </main>
    </div>
  );
}
