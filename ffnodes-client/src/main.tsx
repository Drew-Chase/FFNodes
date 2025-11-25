import React from "react";
import {BrowserRouter, Route, Routes} from "react-router-dom";
import ReactDOM from "react-dom/client";
import $ from "jquery";
import {HeroUIProvider} from "@heroui/react";
import {ToastProvider} from "@heroui/toast";

import "./css/index.css";
import {Setup} from "./pages/Setup.tsx";
import {Dashboard} from "./pages/Dashboard.tsx";
import {Settings} from "./pages/Settings.tsx";
import {TitleBar} from "./components/TitleBar.tsx";

ReactDOM.createRoot($("#root")[0]!).render(
    <HeroUIProvider>
        <ToastProvider placement="top-right" maxVisibleToasts={3} />
        <TitleBar/>
        <main className={"max-h-[calc(100dvh_-_2.5rem)] h-[calc(100dvh_-_2.5rem)] overflow-auto"}>
            <React.StrictMode>
                <BrowserRouter>
                    <Routes>
                        <Route path="/" element={<Setup/>}/>
                        <Route path="/setup" element={<Setup/>}/>
                        <Route path="/dashboard" element={<Dashboard/>}/>
                        <Route path="/settings" element={<Settings/>}/>
                    </Routes>
                </BrowserRouter>
            </React.StrictMode>
        </main>
    </HeroUIProvider>
);
