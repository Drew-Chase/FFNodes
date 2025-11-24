import React from "react";
import { BrowserRouter, Route, Routes } from "react-router-dom";
import ReactDOM from "react-dom/client";
import $ from "jquery";

import "./assets/css/index.css";
import { Setup } from "./assets/pages/Setup.tsx";
import { Dashboard } from "./assets/pages/Dashboard.tsx";
import { Settings } from "./assets/pages/Settings.tsx";

ReactDOM.createRoot($("#root")[0]!).render(
  <React.StrictMode>
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Setup />} />
        <Route path="/setup" element={<Setup />} />
        <Route path="/dashboard" element={<Dashboard />} />
        <Route path="/settings" element={<Settings />} />
      </Routes>
    </BrowserRouter>
  </React.StrictMode>
);
