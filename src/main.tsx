import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import App from "./App";
import { ProjectListPage } from "./pages/ProjectListPage";
import { NewProjectPage } from "./pages/NewProjectPage";
import { ProjectDetailPage } from "./pages/ProjectDetailPage";
import { ScanResultsPage } from "./pages/ScanResultsPage";
import { QuestionnairePage } from "./pages/QuestionnairePage";
import { PlanViewPage } from "./pages/PlanViewPage";
import { SetupPage } from "./pages/SetupPage";
import "./styles/globals.css";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      staleTime: 30_000,
    },
  },
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <Routes>
          <Route element={<App />}>
            <Route index element={<ProjectListPage />} />
            <Route path="setup" element={<SetupPage />} />
            <Route path="projects/new" element={<NewProjectPage />} />
            <Route path="projects/:id" element={<ProjectDetailPage />} />
            <Route
              path="projects/:projectId/scans/:scanId"
              element={<ScanResultsPage />}
            />
            <Route
              path="projects/:id/questionnaire"
              element={<QuestionnairePage />}
            />
            <Route
              path="projects/:projectId/plans/:planId"
              element={<PlanViewPage />}
            />
          </Route>
        </Routes>
      </BrowserRouter>
    </QueryClientProvider>
  </React.StrictMode>
);
