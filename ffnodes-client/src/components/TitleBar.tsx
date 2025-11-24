import {Button, ButtonGroup, cn} from "@heroui/react";
import {getCurrentWindow} from "@tauri-apps/api/window";
import {Icon} from "@iconify-icon/react";

export function TitleBar()
{
    const appWindow = getCurrentWindow();
    return (
        <div
            className={cn(
                "w-full h-[2.5rem] bg-[#191A24FF] items-center justify-between flex flex-row",
                "border-b-primary/20 border-2 border-t-none border-l-none border-r-none border-transparent"
            )}
            style={{userSelect: "none"}}
            data-tauri-drag-region=""
        >
            <p className={"pl-2 text-sm"}>FFProbe</p>
            <ButtonGroup className={"h-full"}>
                <Button variant={"light"} className={"min-w-0 h-full text-[1rem]"} radius={"sm"} onPress={() => appWindow.minimize()}><Icon icon="material-symbols:minimize-rounded"/></Button>
                <Button variant={"light"} className={"min-w-0 h-full text-[.7rem]"} radius={"sm"} onPress={() => appWindow.toggleMaximize()}><Icon icon="material-symbols:square-outline-rounded"/></Button>
                <Button variant={"light"} color={"danger"} className={"min-w-0 h-full text-[1rem]"} radius={"sm"} onPress={() => appWindow.close()}><Icon icon="material-symbols:close-rounded"/></Button>
            </ButtonGroup>
        </div>
    );
}