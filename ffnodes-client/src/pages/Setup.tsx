import {useState} from "react";
import {motion} from "framer-motion";
import {invoke} from "@tauri-apps/api/core";
import {useNavigate} from "react-router-dom";
import toast from "react-hot-toast";
import {NeonCard} from "../components/NeonCard";
import {NeonButton} from "../components/NeonButton";
import {NeonInput} from "../components/NeonInput";

interface FormData
{
    serverUrl: string;
    serverGuid: string;
    displayName: string;
}

interface FormErrors
{
    serverUrl?: string;
    serverGuid?: string;
    displayName?: string;
}

export function Setup()
{
    const navigate = useNavigate();
    const [formData, setFormData] = useState<FormData>({
        serverUrl: "",
        serverGuid: "",
        displayName: ""
    });
    const [errors, setErrors] = useState<FormErrors>({});
    const [isLoading, setIsLoading] = useState(false);
    const [testingConnection, setTestingConnection] = useState(false);

    const validateForm = (): boolean =>
    {
        const newErrors: FormErrors = {};

        if (!formData.serverUrl.trim())
        {
            newErrors.serverUrl = "Server URL is required";
        } else if (!formData.serverUrl.match(/^https?:\/\/.+/))
        {
            newErrors.serverUrl = "Invalid URL format (must start with http:// or https://)";
        }

        if (!formData.serverGuid.trim())
        {
            newErrors.serverGuid = "Server GUID is required";
        }

        if (!formData.displayName.trim())
        {
            newErrors.displayName = "Display name is required";
        }

        setErrors(newErrors);
        return Object.keys(newErrors).length === 0;
    };

    const testConnection = async () =>
    {
        if (!validateForm())
        {
            toast.error("Please fix the errors before testing connection");
            return;
        }

        setTestingConnection(true);

        try
        {
            await invoke("test_connection", {
                input: {
                    server_url: formData.serverUrl,
                    server_guid: formData.serverGuid,
                    display_name: formData.displayName
                }
            });
            toast.success("Connection successful!");
        } catch (error)
        {
            toast.error(`Connection failed: ${error}`);
        } finally
        {
            setTestingConnection(false);
        }
    };

    const handleSubmit = async () =>
    {
        if (!validateForm())
        {
            return;
        }

        setIsLoading(true);

        try
        {
            await invoke("test_connection", {
                input: {
                    server_url: formData.serverUrl,
                    server_guid: formData.serverGuid,
                    display_name: formData.displayName
                }
            });

            toast.success("Setup complete! Redirecting to dashboard...");
            setTimeout(() =>
            {
                navigate("/dashboard");
            }, 1500);
        } catch (error)
        {
            toast.error(`Setup failed: ${error}`);
            setIsLoading(false);
        }
    };

    return (
        <div className="flex items-center justify-center p-8 relative">
            {/* Animated Background */}
            <div className="absolute inset-0 z-0">
                <div className="absolute top-20 left-20 w-96 h-96 bg-[var(--neon-primary)] opacity-10 rounded-full blur-[100px] animate-pulse"/>
                <div className="absolute bottom-20 right-20 w-96 h-96 bg-[var(--neon-accent)] opacity-10 rounded-full blur-[100px] animate-pulse" style={{animationDelay: "1s"}}/>
            </div>

            <div className="w-full max-w-2xl z-10">
                <NeonCard variant="gradient" className="!p-0">
                    <div className="p-12">
                        {/* Header */}
                        <div className="text-center mb-12">
                            <motion.div
                                initial={{y: -20, opacity: 0}}
                                animate={{y: 0, opacity: 1}}
                                transition={{duration: 0.5, delay: 0.2}}
                            >
                                <p className="mb-2 text-2xl font-bold text-white">
                                    Welcome to FFNodes
                                </p>
                                <p className="text-gray-400 mt-4">
                                    Let's get you connected to your encoding server
                                </p>
                            </motion.div>
                        </div>

                        {/* Form */}
                        <div className="space-y-6">
                            <NeonInput
                                label="Server URL"
                                value={formData.serverUrl}
                                onChange={(value) => setFormData({...formData, serverUrl: value})}
                                placeholder="http://192.168.1.100:8080"
                                type="url"
                                error={errors.serverUrl}
                                required
                            />

                            <NeonInput
                                label="Server GUID"
                                value={formData.serverGuid}
                                onChange={(value) => setFormData({...formData, serverGuid: value})}
                                placeholder="Enter the server's unique identifier"
                                error={errors.serverGuid}
                                required
                            />

                            <NeonInput
                                label="Display Name"
                                value={formData.displayName}
                                onChange={(value) => setFormData({...formData, displayName: value})}
                                placeholder="My Encoding Client"
                                error={errors.displayName}
                                required
                            />

                            {/* Info Box */}
                            <motion.div
                                className="glass p-4 rounded-lg border-l-4 border-[var(--neon-accent)]"
                                initial={{opacity: 0}}
                                animate={{opacity: 1}}
                                transition={{delay: 0.5}}
                            >
                                <p className="text-sm text-gray-300">
                                    <span className="neon-text-accent font-bold">Tip:</span> You can find the Server GUID in your FFNodes server's config.json file or in the server logs at startup.
                                </p>
                            </motion.div>
                        </div>

                        {/* Actions */}
                        <div className="flex gap-4 mt-10">
                            <NeonButton
                                variant="accent"
                                onClick={testConnection}
                                loading={testingConnection}
                                disabled={isLoading}
                                className="flex-1"
                            >
                                Test Connection
                            </NeonButton>
                            <NeonButton
                                variant="primary"
                                onClick={handleSubmit}
                                loading={isLoading}
                                disabled={testingConnection}
                                className="flex-1"
                            >
                                Continue
                            </NeonButton>
                        </div>

                        {/* Footer */}
                        <motion.div
                            className="mt-8 text-center"
                            initial={{opacity: 0}}
                            animate={{opacity: 1}}
                            transition={{delay: 0.7}}
                        >
                            <p className="text-xs text-gray-500">
                                This configuration will be saved and you can change it later in settings
                            </p>
                        </motion.div>
                    </div>
                </NeonCard>
            </div>
        </div>
    );
}
