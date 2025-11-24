
import {Button, ButtonGroup} from "@heroui/react";
import {ThemeSwitchComponent} from "../providers/ThemeProvider.tsx";
import {getCurrentWindow} from "@tauri-apps/api/window";
import {Icon} from "@iconify-icon/react";

export default function Navigation()
{
    const appWindow = getCurrentWindow();
    return (

        <div className={"flex flex-row h-[2.5rem] backdrop-blur-sm sticky top-0 w-full z-[51] backdrop-saturate-150 select-none"} data-tauri-drag-region="">
            <div className={"flex flex-row"}>
                <p className={"mx-2 mt-1 text-large font-bold select-none"} data-tauri-drag-region="">FFNodes</p>
            </div>
            <div className={"flex flex-row ml-auto"}>
                <ButtonGroup className={"h-[2rem]"}>
                    <ThemeSwitchComponent/>
                    <Button variant={"light"} className={"min-w-0 h-[2rem] text-[1rem]"} radius={"sm"} onPress={() => appWindow.minimize()}><Icon icon="material-symbols:minimize-rounded"/></Button>
                    <Button variant={"light"} className={"min-w-0 h-[2rem] text-[.7rem]"} radius={"sm"} onPress={() => appWindow.toggleMaximize()}><Icon icon="material-symbols:square-outline-rounded"/></Button>
                    <Button variant={"light"} color={"danger"} className={"min-w-0 h-[2rem] text-[1rem]"} radius={"sm"} onPress={() => appWindow.close()}><Icon icon="material-symbols:close-rounded"/></Button>
                </ButtonGroup>
            </div>
        </div>
    );
}

