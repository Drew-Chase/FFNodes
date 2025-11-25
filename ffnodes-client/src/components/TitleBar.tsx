import {Button, ButtonGroup, cn} from "@heroui/react";
import {getCurrentWindow} from "@tauri-apps/api/window";
import {Icon} from "@iconify-icon/react";
import { useAuthStore } from "../stores/useAuthStore";
import { useLocation } from "react-router-dom";

export function TitleBar()
{
    const appWindow = getCurrentWindow();
    const { user, isAuthenticated } = useAuthStore();
    const location = useLocation();
    const isLoginPage = location.pathname === '/login';

    return (
        <div
            className={cn(
                "w-full h-[2.5rem] bg-content1/50 items-center justify-between flex flex-row shadow-md-2",
                "fixed top-0 z-[50] backdrop-blur-sm backdrop-saturate-150 select-none"
            )}
            style={{userSelect: "none"}}
            data-tauri-drag-region=""
        >
            {/* Left Section - App Title & User Info */}
            <div className="flex items-center gap-4 pl-4 flex-1" data-tauri-drag-region="">
                <div className="flex items-center gap-2">
                    <p className="text-body-md font-medium text-foreground">FFNodes</p>
                </div>

                {!isLoginPage && isAuthenticated && user && (
                    <div className="flex items-center gap-2 ml-4 pl-4 border-l border-divider/30">
                        {user.avatar && (
                            <img
                                src={user.avatar}
                                alt={user.name}
                                className="w-6 h-6 rounded-full"
                            />
                        )}
                        <span className="text-body-sm text-foreground/70">{user.name}</span>
                    </div>
                )}
            </div>

            {/* Right Section - Window Controls */}
            <ButtonGroup className="h-full">
                <Button
                    variant="light"
                    className="min-w-0 h-full text-[1rem] rounded-none hover:bg-content2"
                    radius="none"
                    onPress={() => appWindow.minimize()}
                >
                    <Icon icon="material-symbols:minimize-rounded"/>
                </Button>
                <Button
                    variant="light"
                    className="min-w-0 h-full text-[.7rem] rounded-none hover:bg-content2"
                    radius="none"
                    onPress={() => appWindow.toggleMaximize()}
                >
                    <Icon icon="material-symbols:square-outline-rounded"/>
                </Button>
                <Button
                    variant="light"
                    color="danger"
                    className="min-w-0 h-full text-[1rem] rounded-none hover:bg-danger/10"
                    radius="none"
                    onPress={() => appWindow.close()}
                >
                    <Icon icon="material-symbols:close-rounded"/>
                </Button>
            </ButtonGroup>
        </div>
    );
}