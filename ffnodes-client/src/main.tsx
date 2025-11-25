import React, {useEffect} from "react";
import {BrowserRouter, Route, Routes, Navigate} from "react-router-dom";
import ReactDOM from "react-dom/client";
import $ from "jquery";
import {HeroUIProvider} from "@heroui/react";
import {ToastProvider} from "@heroui/toast";

import "./css/index.css";
import {Setup} from "./pages/Setup.tsx";
import {Dashboard} from "./pages/Dashboard.tsx";
import {Settings} from "./pages/Settings.tsx";
import {Login} from "./pages/Login.tsx";
import {TitleBar} from "./components/TitleBar.tsx";
import {ProtectedRoute} from "./components/auth/ProtectedRoute.tsx";
import {useAuthStore} from "./stores/useAuthStore.ts";
import {useThemeStore} from "./stores/useThemeStore.ts";

function App() {
    const { isAuthenticated } = useAuthStore();
    const { theme } = useThemeStore();

    // Apply theme on mount and when it changes
    useEffect(() => {
        if (theme === 'dark') {
            document.documentElement.classList.add('dark');
        } else {
            document.documentElement.classList.remove('dark');
        }
    }, [theme]);

    return (
        <HeroUIProvider>
            <ToastProvider placement="bottom-right" maxVisibleToasts={3} />
            <BrowserRouter>
                <TitleBar/>
                <main className={"max-h-screen h-screen overflow-auto"}>
                    <Routes>
                        <Route path="/" element={isAuthenticated ? <Navigate to="/dashboard" replace /> : <Navigate to="/login" replace />} />
                        <Route path="/login" element={<ProtectedRoute requireAuth={false}><Login /></ProtectedRoute>} />
                        <Route path="/setup" element={<ProtectedRoute><Setup /></ProtectedRoute>} />
                        <Route path="/dashboard" element={<ProtectedRoute><Dashboard /></ProtectedRoute>} />
                        <Route path="/settings" element={<ProtectedRoute><Settings /></ProtectedRoute>} />
                    </Routes>
                </main>
            </BrowserRouter>
        </HeroUIProvider>
    );
}

ReactDOM.createRoot($("#root")[0]!).render(
    <React.StrictMode>
        <App />
    </React.StrictMode>
);
