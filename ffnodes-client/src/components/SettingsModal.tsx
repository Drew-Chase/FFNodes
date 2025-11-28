import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import {addToast} from "@heroui/toast";
import {ClientConfig, useConfigStore} from "../stores/useConfigStore";
import {Button, Input} from "./ui";
import {BentoGrid, BentoCard, BentoCardHeader, BentoCardContent, BentoCardFooter} from "./layout/BentoGrid";
import {useAuthStore} from "../stores/useAuthStore";
import {OAuthService} from "../services/oauth";
import {useThemeStore} from "../stores/useThemeStore";
import {Modal, ModalContent, ModalHeader, ModalBody, ModalFooter} from "@heroui/react";

interface FormData
{
    serverUrl: string;
    serverGuid: string;
    displayName: string;
}

interface GpuInfo
{
    vendor: string;
    name: string;
    encoder_h264: string;
    encoder_h265: string;
}

interface SettingsModalProps
{
    isOpen: boolean;
    onClose: () => void;
}

export function SettingsModal({isOpen, onClose}: SettingsModalProps)
{
    const {config, setConfig} = useConfigStore();
    const {theme, setTheme} = useThemeStore();
    const [formData, setFormData] = useState<FormData>({
        serverUrl: config?.server_url || "",
        serverGuid: config?.server_guid || "",
        displayName: config?.display_name || ""
    });
    const [gpuInfo, setGpuInfo] = useState<GpuInfo | null>(null);
    const [isTesting, setIsTesting] = useState(false);
    const [isSaving, setIsSaving] = useState(false);
    const [connectionStatus, setConnectionStatus] = useState<"idle" | "success" | "error">("idle");
    const [ffmpegCommand, setFfmpegCommand] = useState<string | null>(null);

    useEffect(() =>
    {
        if (isOpen)
        {
            // Load GPU info when modal opens
            const loadGpuInfo = async () =>
            {
                try
                {
                    const gpu = await invoke<GpuInfo>("get_gpu_info");
                    setGpuInfo(gpu);
                } catch (error)
                {
                    console.error("Failed to load GPU info:", error);
                }
            };
            loadGpuInfo();
        }
    }, [isOpen]);

    useEffect(() =>
    {
        // Load FFmpeg command when config and GPU info are available
        const loadFfmpegCommand = async () =>
        {
            if (config && gpuInfo && config.ffmpeg_template)
            {
                try
                {
                    const command = await invoke<string>("get_ffmpeg_command", {
                        config,
                        gpu: gpuInfo
                    });
                    setFfmpegCommand(command);
                } catch (error)
                {
                    console.error("Failed to load FFmpeg command:", error);
                }
            }
        };
        loadFfmpegCommand();
    }, [config, gpuInfo]);

    // Update form data when config changes
    useEffect(() =>
    {
        if (config)
        {
            setFormData({
                serverUrl: config.server_url || "",
                serverGuid: config.server_guid || "",
                displayName: config.display_name || ""
            });
        }
    }, [config]);

    const handleInputChange = (field: keyof FormData) => (value: string) =>
    {
        setFormData((prev) => ({...prev, [field]: value}));
        setConnectionStatus("idle"); // Reset status on change
    };

    const handleTestConnection = async () =>
    {
        if (!formData.serverUrl || !formData.serverGuid || !formData.displayName)
        {
            addToast({
                title: "Error",
                description: "All fields are required",
                color: "danger"
            });
            return;
        }

        setIsTesting(true);
        setConnectionStatus("idle");

        try
        {
            await invoke("test_connection", {
                input: {
                    server_url: formData.serverUrl,
                    server_guid: formData.serverGuid,
                    display_name: formData.displayName
                }
            });

            setConnectionStatus("success");
            addToast({
                title: "Success",
                description: "Connection successful!",
                color: "success"
            });
        } catch (error)
        {
            setConnectionStatus("error");
            addToast({
                title: "Error",
                description: `Connection failed: ${error}`,
                color: "danger",
                timeout: 6000
            });
        } finally
        {
            setIsTesting(false);
        }
    };

    const handleSave = async () =>
    {
        if (!formData.serverUrl || !formData.serverGuid || !formData.displayName)
        {
            addToast({
                title: "Error",
                description: "All fields are required",
                color: "danger"
            });
            return;
        }

        setIsSaving(true);

        try
        {
            const newConfig: ClientConfig = await invoke("test_connection", {
                input: {
                    server_url: formData.serverUrl,
                    server_guid: formData.serverGuid,
                    display_name: formData.displayName
                }
            });

            setConfig(newConfig);
            addToast({
                title: "Success",
                description: "Settings saved successfully!",
                color: "success"
            });

            // Close modal after saving
            setTimeout(() =>
            {
                onClose();
            }, 1000);
        } catch (error)
        {
            addToast({
                title: "Error",
                description: `Failed to save: ${error}`,
                color: "danger",
                timeout: 6000
            });
        } finally
        {
            setIsSaving(false);
        }
    };

    const {user, isAuthenticated} = useAuthStore();

    const handleLogout = () =>
    {
        OAuthService.logout();
        onClose();
    };

    return (
        <Modal
            isOpen={isOpen}
            onOpenChange={onClose}
            size="5xl"
            backdrop="blur"
            scrollBehavior="inside"
        >
            <ModalContent>
                {(onClose) => (
                    <>
                        <ModalHeader className="flex flex-col gap-1">
                            <h2 className="text-headline-lg font-medium text-primary">Settings</h2>
                            <p className="text-body-sm text-foreground/70">Configure your FFNodes client</p>
                        </ModalHeader>
                        <ModalBody>
                            <BentoGrid columns={6} gap="md">
                                {/* Server Configuration - Large Card */}
                                <BentoCard
                                    colSpan={4}
                                    elevation={3}
                                    background="glass"
                                >
                                    <BentoCardHeader
                                        title="Server Configuration"
                                        subtitle="Configure connection to your FFNodes server"
                                        icon={<iconify-icon icon="mdi:server" class="text-2xl"/>}
                                    />
                                    <BentoCardContent>
                                        <div className="space-y-4 mt-4">
                                            <Input
                                                label="Server URL"
                                                placeholder="http://localhost:8080"
                                                value={formData.serverUrl}
                                                onChange={handleInputChange("serverUrl")}
                                                required
                                            />

                                            <Input
                                                label="Server GUID"
                                                placeholder="Enter server secret code"
                                                value={formData.serverGuid}
                                                onChange={handleInputChange("serverGuid")}
                                                required
                                            />

                                            <Input
                                                label="Display Name"
                                                placeholder="My Encoding Client"
                                                value={formData.displayName}
                                                onChange={handleInputChange("displayName")}
                                                required
                                            />
                                        </div>
                                    </BentoCardContent>
                                    <BentoCardFooter>
                                        <div className="flex gap-3 items-center">
                                            <Button
                                                onClick={handleTestConnection}
                                                loading={isTesting}
                                                isDisabled={isTesting || !formData.serverUrl || !formData.serverGuid || !formData.displayName}
                                                color="secondary"
                                                className="rounded-md-lg"
                                            >
                                                Test Connection
                                            </Button>

                                            {connectionStatus === "success" && (
                                                <div className="flex items-center gap-2">
                                                    <div className="w-2 h-2 rounded-full bg-success animate-pulse"/>
                                                    <span className="text-success text-body-sm font-medium">Connected</span>
                                                </div>
                                            )}

                                            {connectionStatus === "error" && (
                                                <div className="flex items-center gap-2">
                                                    <div className="w-2 h-2 rounded-full bg-danger"/>
                                                    <span className="text-danger text-body-sm font-medium">Failed</span>
                                                </div>
                                            )}
                                        </div>
                                    </BentoCardFooter>
                                </BentoCard>

                                {/* Account Info */}
                                <BentoCard
                                    colSpan={2}
                                    elevation={3}
                                    background="gradient"
                                >
                                    <BentoCardHeader
                                        title="Account"
                                        subtitle={isAuthenticated ? "Logged in" : "Not logged in"}
                                        icon={<iconify-icon icon="mdi:account-circle" class="text-2xl"/>}
                                    />
                                    <BentoCardContent>
                                        {user ? (
                                            <div className="mt-4 space-y-3">
                                                {user.avatar && (
                                                    <img
                                                        src={user.avatar}
                                                        alt={user.name}
                                                        className="w-16 h-16 rounded-full"
                                                    />
                                                )}
                                                <div>
                                                    <p className="text-title-md font-medium text-foreground">{user.name}</p>
                                                    {user.email && (
                                                        <p className="text-body-sm text-foreground/70">{user.email}</p>
                                                    )}
                                                    <p className="text-body-sm text-foreground/50 mt-1">
                                                        via {user.provider}
                                                    </p>
                                                </div>
                                            </div>
                                        ) : (
                                            <p className="text-body-sm text-foreground/70 mt-4">
                                                No account logged in
                                            </p>
                                        )}
                                    </BentoCardContent>
                                    {user && (
                                        <BentoCardFooter>
                                            <Button
                                                onPress={handleLogout}
                                                color="danger"
                                                size="sm"
                                                className="rounded-md-sm"
                                            >
                                                Logout
                                            </Button>
                                        </BentoCardFooter>
                                    )}
                                </BentoCard>

                                {/* GPU Information */}
                                <BentoCard
                                    colSpan={3}
                                    elevation={2}
                                    background="glass"
                                >
                                    <BentoCardHeader
                                        title="GPU Information"
                                        subtitle="Hardware acceleration details"
                                        icon={<iconify-icon icon="mdi:chip" class="text-2xl"/>}
                                    />
                                    <BentoCardContent>
                                        {gpuInfo ? (
                                            <div className="mt-4 grid grid-cols-2 gap-4">
                                                <div>
                                                    <p className="text-body-sm text-foreground/60 mb-1">Vendor</p>
                                                    <p className="text-body-md font-medium text-foreground">{gpuInfo.vendor}</p>
                                                </div>
                                                <div>
                                                    <p className="text-body-sm text-foreground/60 mb-1">GPU Model</p>
                                                    <p className="text-body-md font-medium text-foreground">{gpuInfo.name}</p>
                                                </div>
                                                <div>
                                                    <p className="text-body-sm text-foreground/60 mb-1">H.264 Encoder</p>
                                                    <p className="text-body-sm font-mono text-secondary">{gpuInfo.encoder_h264}</p>
                                                </div>
                                                <div>
                                                    <p className="text-body-sm text-foreground/60 mb-1">H.265 Encoder</p>
                                                    <p className="text-body-sm font-mono text-secondary">{gpuInfo.encoder_h265}</p>
                                                </div>
                                            </div>
                                        ) : (
                                            <div className="mt-4 flex items-center justify-center h-24">
                                                <div className="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-primary"></div>
                                            </div>
                                        )}
                                    </BentoCardContent>
                                </BentoCard>

                                {/* Current Session */}
                                <BentoCard
                                    colSpan={3}
                                    elevation={2}
                                    background="glass"
                                >
                                    <BentoCardHeader
                                        title="Current Session"
                                        subtitle="Active client information"
                                        icon={<iconify-icon icon="mdi:information-outline" class="text-2xl"/>}
                                    />
                                    <BentoCardContent>
                                        {config ? (
                                            <div className="mt-4 space-y-3">
                                                <div className="flex justify-between items-center">
                                                    <span className="text-body-sm text-foreground/60">Client ID</span>
                                                    <span className="text-body-sm font-mono text-foreground">{config.client_id || "Not set"}</span>
                                                </div>
                                                <div className="flex justify-between items-center">
                                                    <span className="text-body-sm text-foreground/60">Computer Name</span>
                                                    <span className="text-body-sm text-foreground">{config.computer_name}</span>
                                                </div>
                                                <div className="flex justify-between items-center">
                                                    <span className="text-body-sm text-foreground/60">Auth Status</span>
                                                    <span className={`text-body-sm font-medium ${config.auth_token ? "text-success" : "text-foreground/60"}`}>
                            {config.auth_token ? "Authenticated" : "Not authenticated"}
                          </span>
                                                </div>
                                                {ffmpegCommand && (
                                                    <div className="pt-3 border-t border-foreground/10">
                                                        <p className="text-body-sm text-foreground/60 mb-2">FFmpeg Command</p>
                                                        <div className="bg-content2 p-3 rounded-md-sm overflow-x-auto">
                                                            <code className="text-body-xs font-mono text-foreground break-all whitespace-pre-wrap">
                                                                {ffmpegCommand}
                                                            </code>
                                                        </div>
                                                    </div>
                                                )}
                                            </div>
                                        ) : (
                                            <p className="text-body-sm text-foreground/70 mt-4">No active session</p>
                                        )}
                                    </BentoCardContent>
                                </BentoCard>

                                {/* Theme Settings */}
                                <BentoCard
                                    colSpan={3}
                                    elevation={2}
                                    background={"gradient"}
                                >
                                    <BentoCardHeader
                                        title="Appearance"
                                        subtitle="Theme preferences"
                                        icon={<iconify-icon icon="mdi:palette" class="text-2xl"/>}
                                    />
                                    <BentoCardContent>
                                        <div className="mt-4 flex gap-2">
                                            <button
                                                onClick={() => setTheme("light")}
                                                className={`flex-1 p-3 rounded-md-sm transition-colors ${
                                                    theme === "light"
                                                        ? "bg-primary text-primary-foreground"
                                                        : "bg-content2 hover:bg-content3"
                                                }`}
                                            >
                                                <iconify-icon icon="mdi:white-balance-sunny" class="text-xl"/>
                                                <p className="text-body-sm mt-1">Light</p>
                                            </button>
                                            <button
                                                onClick={() => setTheme("dark")}
                                                className={`flex-1 p-3 rounded-md-sm transition-colors ${
                                                    theme === "dark"
                                                        ? "bg-primary text-primary-foreground"
                                                        : "bg-content2 hover:bg-content3"
                                                }`}
                                            >
                                                <iconify-icon icon="mdi:moon-waning-crescent" class="text-xl"/>
                                                <p className="text-body-sm mt-1">Dark</p>
                                            </button>
                                        </div>
                                    </BentoCardContent>
                                </BentoCard>

                                {/* About */}
                                <BentoCard
                                    colSpan={3}
                                    elevation={1}
                                    background={"solid"}
                                >
                                    <BentoCardHeader
                                        title="About"
                                        icon={<iconify-icon icon="mdi:information" class="text-2xl"/>}
                                    />
                                    <BentoCardContent>
                                        <div className="mt-4 space-y-2">
                                            <p className="text-body-sm text-foreground/70">FFNodes Client</p>
                                            <p className="text-body-sm text-foreground/60">Version 0.1.0</p>
                                            <p className="text-body-sm text-foreground/50">Built with Tauri & React</p>
                                        </div>
                                    </BentoCardContent>
                                </BentoCard>
                            </BentoGrid>
                        </ModalBody>
                        <ModalFooter>
                            <Button
                                onPress={handleSave}
                                loading={isSaving}
                                isDisabled={isSaving || !formData.serverUrl || !formData.serverGuid || !formData.displayName}
                                color="primary"
                            >
                                <iconify-icon icon="mdi:content-save" class="text-xl mr-2"/>
                                Save & Apply Changes
                            </Button>
                            <Button color="danger" onPress={onClose} >
                                Cancel
                            </Button>
                        </ModalFooter>
                    </>
                )}
            </ModalContent>
        </Modal>
    );
}
