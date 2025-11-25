import {useState} from "react";
import {motion} from "framer-motion";
import {invoke} from "@tauri-apps/api/core";
import {useNavigate} from "react-router-dom";
import {addToast} from "@heroui/toast";
import {Button, Input, Card} from "../components/ui";

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
            addToast({
                title: "Error",
                description: "Please fix the errors before testing connection",
                color: "danger"
            });
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
            addToast({
                title: "Success",
                description: "Connection successful!",
                color: "success"
            });
        } catch (error)
        {
            addToast({
                title: "Error",
                description: `Connection failed: ${error}`,
                color: "danger"
            });
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

            addToast({
                title: "Success",
                description: "Setup complete! Redirecting to dashboard...",
                color: "success"
            });
            setTimeout(() =>
            {
                navigate("/dashboard");
            }, 1500);
        } catch (error)
        {
            addToast({
                title: "Error",
                description: `Setup failed: ${error}`,
                color: "danger"
            });
            setIsLoading(false);
        }
    };

    return (
        <div className="flex items-center justify-center p-8 relative">
            <div className="w-full max-w-2xl z-10">
                <Card variant="gradient" className="!p-0">
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
                            <Input
                                label="Server URL"
                                value={formData.serverUrl}
                                onChange={(value) => setFormData({...formData, serverUrl: value})}
                                placeholder="http://192.168.1.100:8080"
                                type="url"
                                error={errors.serverUrl}
                                required
                            />

                            <Input
                                label="Server GUID"
                                value={formData.serverGuid}
                                onChange={(value) => setFormData({...formData, serverGuid: value})}
                                placeholder="Enter the server's unique identifier"
                                error={errors.serverGuid}
                                required
                            />

                            <Input
                                label="Display Name"
                                value={formData.displayName}
                                onChange={(value) => setFormData({...formData, displayName: value})}
                                placeholder="My Encoding Client"
                                error={errors.displayName}
                                required
                            />

                            {/* Info Box */}
                            <motion.div
                                className="bg-default-100 p-4 rounded-lg border-l-4 border-primary"
                                initial={{opacity: 0}}
                                animate={{opacity: 1}}
                                transition={{delay: 0.5}}
                            >
                                <p className="text-sm text-default-700">
                                    <span className="text-primary font-bold">Tip:</span> You can find the Server GUID in your FFNodes server's config.json file or in the server logs at startup.
                                </p>
                            </motion.div>
                        </div>

                        {/* Actions */}
                        <div className="flex gap-4 mt-10">
                            <Button
                                variant="accent"
                                onClick={testConnection}
                                loading={testingConnection}
                                isDisabled={isLoading}
                                className="flex-1"
                            >
                                Test Connection
                            </Button>
                            <Button
                                variant="primary"
                                onClick={handleSubmit}
                                loading={isLoading}
                                isDisabled={testingConnection}
                                className="flex-1"
                            >
                                Continue
                            </Button>
                        </div>

                        {/* Footer */}
                        <motion.div
                            className="mt-8 text-center"
                            initial={{opacity: 0}}
                            animate={{opacity: 1}}
                            transition={{delay: 0.7}}
                        >
                            <p className="text-xs text-default-500">
                                This configuration will be saved and you can change it later in settings
                            </p>
                        </motion.div>
                    </div>
                </Card>
            </div>
        </div>
    );
}
