import React from "react";
import ReactDOM from "react-dom/client";
import {HeroUIProvider} from "@heroui/react";
import {Dashboard} from "./components/Dashboard";
import {ThemeProvider, useTheme} from "./contexts/ThemeContext";
import "./css/index.css";

function App()
{
    const {theme} = useTheme();

    return (
        <HeroUIProvider className={theme}>
            <Dashboard/>
        </HeroUIProvider>
    );
}

ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
        <ThemeProvider>
            <App/>
        </ThemeProvider>
    </React.StrictMode>
);
