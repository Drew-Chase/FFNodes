import React, {useState} from "react";
import {useNavigate} from "react-router-dom";
import {AnimatePresence, motion} from "framer-motion";
import {Button, Input} from "../components/ui";
import {MoviePosterBackground} from "../components/MoviePosterScroll";
import {OAuthService} from "../services/oauth";
import {QRCodeSVG} from "qrcode.react";
import {invoke} from "@tauri-apps/api/core";
import {addToast} from "@heroui/toast";
import {ClientConfig, useConfigStore} from "../stores/useConfigStore";

type LoginStep = 'auth' | 'display-name' | 'server-setup';

export const Login: React.FC = () =>
{
    const navigate = useNavigate();
    const {setConfig} = useConfigStore();

    // State
    const [step, setStep] = useState<LoginStep>('display-name');
    const [email, setEmail] = useState("");
    const [displayName, setDisplayName] = useState("");
    const [serverUrl, setServerUrl] = useState("");
    const [serverGuid, setServerGuid] = useState("");
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState("");
    const [showQR, setShowQR] = useState(false);
    const [authProvider, setAuthProvider] = useState<string | null>(null);
    const [connectionStatus, setConnectionStatus] = useState<'idle' | 'testing' | 'success' | 'error'>('idle');

    const handleOAuthLogin = async (provider: "google" | "github" | "microsoft" | "facebook") =>
    {
        setLoading(true);
        setError("");

        try
        {
            await OAuthService.login(provider);
            setAuthProvider(provider);
            // Move to server setup step
            setStep('server-setup');
        } catch (err)
        {
            setError(`Failed to login with ${provider}. Please try again.`);
            console.error(err);
        } finally
        {
            setLoading(false);
        }
    };

    const handleEmailLogin = async () =>
    {
        if (!email)
        {
            setError("Please enter your email address");
            return;
        }

        setError("");
        setDisplayName(email);
        setAuthProvider('email');
        // Move to server setup step
        setStep('server-setup');
    };

    const handleDisplayNameContinue = () =>
    {
        if (!displayName.trim())
        {
            setError("Please enter a display name");
            return;
        }

        setError("");
        setAuthProvider('display-name');
        // Move to server setup step
        setStep('server-setup');
    };

    const handleServerSetup = async () =>
    {
        if (!serverUrl || !serverGuid)
        {
            setError("Please enter both server URL and GUID");
            return;
        }

        const finalDisplayName = displayName || email || "Anonymous User";

        setLoading(true);
        setConnectionStatus('testing');
        setError("");

        try
        {
            // Test connection and save config
            const config = await invoke<ClientConfig>('test_connection', {
                input: {
                    server_url: serverUrl,
                    server_guid: serverGuid,
                    display_name: finalDisplayName,
                },
            });

            setConfig(config);
            setConnectionStatus('success');

            // Save authentication state if using display name
            if (authProvider === 'display-name' || authProvider === 'email')
            {
                OAuthService.loginWithDisplayName(finalDisplayName);
            }

            addToast({
                title: 'Success',
                description: 'Connected to server successfully!',
                color: 'success'
            });

            // Navigate to dashboard
            setTimeout(() => navigate("/dashboard"), 500);
        } catch (err)
        {
            setConnectionStatus('error');
            setError(`Failed to connect: ${err}`);
            console.error(err);
        } finally
        {
            setLoading(false);
        }
    };

    // Generate QR code URL for mobile login
    const qrCodeUrl = typeof window !== "undefined" ? window.location.href : "";

    return (
        <MoviePosterBackground>
            <div className="flex items-center justify-center min-h-screen p-4">
                <AnimatePresence mode="wait">
                    {step === 'auth' && (
                        <motion.div
                            key="oauth-login"
                            initial={{opacity: 0, scale: 0.95}}
                            animate={{opacity: 1, scale: 1}}
                            exit={{opacity: 0, scale: 0.95}}
                            transition={{duration: 0.3}}
                            className="relative w-full max-w-md"
                        >
                            {/* Login Modal */}
                            <div className="bg-content1/95 backdrop-blur-sm rounded-md-2xl shadow-md-6 p-8 relative max-h-[calc(100dvh_-_200px)] overflow-y-auto">
                                {/* Close button (optional) */}
                                <button
                                    onClick={() => navigate("/dashboard")}
                                    className="absolute top-4 right-4 text-foreground/60 hover:text-foreground transition-colors"
                                >
                                    <iconify-icon icon="mdi:close" class="text-2xl"/>
                                </button>

                                {/* Header */}
                                <div className="text-center mb-8">
                                    <h1 className="text-display-sm font-normal text-foreground mb-2">
                                        Sign into FFNodes
                                    </h1>
                                    <p className="text-body-md text-foreground/70">
                                        Choose your preferred login method
                                    </p>
                                </div>

                                {/* OAuth Buttons */}
                                <div className="space-y-3 mb-6">
                                    <Button
                                        onPress={() => handleOAuthLogin("google")}
                                        disabled={loading}
                                        className="w-full bg-white hover:bg-gray-50 text-gray-800 border border-gray-300 h-12 rounded-md-lg font-medium shadow-md-2 transition-all duration-md-medium-2"
                                    >
                                        <div className="flex items-center justify-center gap-3">
                                            <iconify-icon icon="logos:google-icon" class="text-xl"/>
                                            <span>Sign in with Google</span>
                                        </div>
                                    </Button>

                                    <Button
                                        onPress={() => handleOAuthLogin("github")}
                                        disabled={loading}
                                        className="w-full bg-white hover:bg-gray-50 text-gray-800 border border-gray-300 h-12 rounded-md-lg font-medium shadow-md-2 transition-all duration-md-medium-2"
                                    >
                                        <div className="flex items-center justify-center gap-3">
                                            <iconify-icon icon="mdi:github" class="text-xl"/>
                                            <span>Log in with GitHub</span>
                                        </div>
                                    </Button>

                                    <Button
                                        onPress={() => handleOAuthLogin("microsoft")}
                                        disabled={loading}
                                        className="w-full bg-white hover:bg-gray-50 text-gray-800 border border-gray-300 h-12 rounded-md-lg font-medium shadow-md-2 transition-all duration-md-medium-2"
                                    >
                                        <div className="flex items-center justify-center gap-3">
                                            <iconify-icon icon="logos:microsoft-icon" class="text-xl"/>
                                            <span>Log in with Microsoft</span>
                                        </div>
                                    </Button>

                                    <Button
                                        onPress={() => handleOAuthLogin("facebook")}
                                        disabled={loading}
                                        className="w-full bg-white hover:bg-gray-50 text-gray-800 border border-gray-300 h-12 rounded-md-lg font-medium shadow-md-2 transition-all duration-md-medium-2"
                                    >
                                        <div className="flex items-center justify-center gap-3">
                                            <iconify-icon icon="logos:facebook" class="text-xl"/>
                                            <span>Log in with Facebook</span>
                                        </div>
                                    </Button>
                                </div>

                                {/* Divider */}
                                <div className="relative my-6">
                                    <div className="absolute inset-0 flex items-center">
                                        <div className="w-full border-t border-divider"/>
                                    </div>
                                    <div className="relative flex justify-center text-body-sm">
                                        <span className="px-4 bg-content1 text-foreground/60">or</span>
                                    </div>
                                </div>

                                {/* Email Input */}
                                <div className="space-y-4 mb-6">
                                    <Input
                                        type="email"
                                        placeholder="email@example.com"
                                        value={email}
                                        onChange={(value) => setEmail(value)}
                                        className="w-full"
                                        disabled={loading}
                                    />
                                    <Button
                                        onPress={handleEmailLogin}
                                        disabled={loading || !email}
                                        className="w-full bg-content3 hover:bg-content4 text-foreground h-12 rounded-md-lg font-medium transition-all duration-md-medium-2"
                                    >
                                        Log in with email
                                    </Button>
                                </div>

                                {/* QR Code Section */}
                                {showQR ? (
                                    <motion.div
                                        initial={{opacity: 0, height: 0}}
                                        animate={{opacity: 1, height: "auto"}}
                                        className="mb-6 text-center"
                                    >
                                        <div className="inline-block p-4 bg-white rounded-md-lg">
                                            <QRCodeSVG value={qrCodeUrl} size={200}/>
                                        </div>
                                        <p className="text-body-sm text-foreground/70 mt-3">
                                            Scan with your iOS camera
                                        </p>
                                        <button
                                            onClick={() => setShowQR(false)}
                                            className="text-primary hover:underline text-body-sm mt-2"
                                        >
                                            Hide QR code
                                        </button>
                                    </motion.div>
                                ) : (
                                    <button
                                        onClick={() => setShowQR(true)}
                                        className="w-full text-center text-body-md text-foreground/70 hover:text-foreground transition-colors mb-6"
                                    >
                                        Or scan QR code with your iOS camera
                                    </button>
                                )}

                                {/* Error Message */}
                                {error && (
                                    <motion.div
                                        initial={{opacity: 0, y: -10}}
                                        animate={{opacity: 1, y: 0}}
                                        className="mb-4 p-3 bg-danger/10 border border-danger rounded-md-sm text-danger text-body-sm"
                                    >
                                        {error}
                                    </motion.div>
                                )}

                                {/* Skip with Display Name */}
                                <button
                                    onClick={() => setStep('display-name')}
                                    className="w-full text-center text-body-md text-foreground/60 hover:text-primary transition-colors underline"
                                >
                                    Skip and use display name
                                </button>

                                {/* Footer */}
                                <div className="mt-6 text-center">
                                    <button className="text-body-sm text-foreground/70 hover:text-foreground hover:underline transition-colors">
                                        Already have an account? Sign in
                                    </button>
                                </div>
                            </div>

                            {/* FFNodes branding */}
                            <div className="mt-4 text-center text-body-sm text-white/70">
                                <span>Powered by FFNodes</span>
                            </div>
                        </motion.div>
                    )}

                    {step === 'display-name' && (
                        <motion.div
                            key="display-name-login"
                            initial={{opacity: 0, scale: 0.95}}
                            animate={{opacity: 1, scale: 1}}
                            exit={{opacity: 0, scale: 0.95}}
                            transition={{duration: 0.3}}
                            className="relative w-full max-w-md"
                        >
                            {/* Display Name Login */}
                            <div className="bg-content1/95 backdrop-blur-xl rounded-md-2xl shadow-md-6 p-8">
                                {/*<button*/}
                                {/*    onClick={() => setStep('auth')}*/}
                                {/*    className="mb-4 text-foreground/60 hover:text-foreground transition-colors flex items-center gap-2"*/}
                                {/*>*/}
                                {/*    <iconify-icon icon="mdi:arrow-left" class="text-xl"/>*/}
                                {/*    <span className="text-body-md">Back to OAuth</span>*/}
                                {/*</button>*/}

                                <div className="text-center mb-8">
                                    <h1 className="text-headline-lg font-normal text-foreground mb-2">
                                        Quick Login
                                    </h1>
                                    <p className="text-body-md text-foreground/70">
                                        Enter a display name to continue
                                    </p>
                                </div>

                                <div className="space-y-4">
                                    <Input
                                        type="text"
                                        placeholder="Enter your display name"
                                        value={displayName}
                                        onChange={(value) => setDisplayName(value)}
                                        className="w-full"
                                    />
                                    <Button
                                        onPress={handleDisplayNameContinue}
                                        disabled={!displayName.trim()}
                                        className="w-full h-12 rounded-md-lg font-medium"
                                    >
                                        Continue
                                    </Button>
                                </div>

                                {error && (
                                    <motion.div
                                        initial={{opacity: 0, y: -10}}
                                        animate={{opacity: 1, y: 0}}
                                        className="mt-4 p-3 bg-danger/10 border border-danger rounded-md-sm text-danger text-body-sm"
                                    >
                                        {error}
                                    </motion.div>
                                )}

                                {/* FFNodes branding */}
                                <div className="mt-4 text-center text-body-sm text-white/70">
                                    <span>Powered by FFNodes</span>
                                </div>
                            </div>
                        </motion.div>
                    )}

                    {step === 'server-setup' && (
                        <motion.div
                            key="server-setup"
                            initial={{opacity: 0, scale: 0.95}}
                            animate={{opacity: 1, scale: 1}}
                            exit={{opacity: 0, scale: 0.95}}
                            transition={{duration: 0.3}}
                            className="relative w-full max-w-md"
                        >
                            {/* Server Setup */}
                            <div className="bg-content1/95 backdrop-blur-xl rounded-md-2xl shadow-md-6 p-8">
                                <button
                                    onClick={() => setStep(authProvider === 'display-name' ? 'display-name' : 'auth')}
                                    className="mb-4 text-foreground/60 hover:text-foreground transition-colors flex items-center gap-2"
                                    disabled={loading}
                                >
                                    <iconify-icon icon="mdi:arrow-left" class="text-xl"/>
                                    <span className="text-body-md">Back</span>
                                </button>

                                <div className="text-center mb-8">
                                    <h1 className="text-headline-lg font-normal text-foreground mb-2">
                                        Connect to Server
                                    </h1>
                                    <p className="text-body-md text-foreground/70">
                                        Enter your server details to continue
                                    </p>
                                </div>

                                <div className="space-y-4">
                                    <Input
                                        type="text"
                                        placeholder="Server URL (e.g., http://localhost:8080)"
                                        value={serverUrl}
                                        onChange={(value) => setServerUrl(value)}
                                        className="w-full"
                                        disabled={loading}
                                    />
                                    <Input
                                        type="text"
                                        placeholder="Server GUID"
                                        value={serverGuid}
                                        onChange={(value) => setServerGuid(value)}
                                        className="w-full"
                                        disabled={loading}
                                    />

                                    {/* Connection Status Indicator */}
                                    {connectionStatus !== 'idle' && (
                                        <motion.div
                                            initial={{opacity: 0, y: -10}}
                                            animate={{opacity: 1, y: 0}}
                                            className={`p-3 rounded-md-sm text-body-sm flex items-center gap-2 ${
                                                connectionStatus === 'testing' ? 'bg-warning/10 border border-warning text-warning' :
                                                connectionStatus === 'success' ? 'bg-success/10 border border-success text-success' :
                                                'bg-danger/10 border border-danger text-danger'
                                            }`}
                                        >
                                            {connectionStatus === 'testing' && (
                                                <>
                                                    <div className="animate-spin rounded-full h-4 w-4 border-t-2 border-b-2 border-warning"></div>
                                                    <span>Testing connection...</span>
                                                </>
                                            )}
                                            {connectionStatus === 'success' && (
                                                <>
                                                    <iconify-icon icon="mdi:check-circle" class="text-lg"/>
                                                    <span>Connected successfully!</span>
                                                </>
                                            )}
                                            {connectionStatus === 'error' && (
                                                <>
                                                    <iconify-icon icon="mdi:alert-circle" class="text-lg"/>
                                                    <span>Connection failed</span>
                                                </>
                                            )}
                                        </motion.div>
                                    )}

                                    <Button
                                        onPress={handleServerSetup}
                                        disabled={loading || !serverUrl || !serverGuid}
                                        className="w-full h-12 rounded-md-lg font-medium"
                                    >
                                        {loading ? 'Connecting...' : 'Connect to Server'}
                                    </Button>
                                </div>

                                {error && (
                                    <motion.div
                                        initial={{opacity: 0, y: -10}}
                                        animate={{opacity: 1, y: 0}}
                                        className="mt-4 p-3 bg-danger/10 border border-danger rounded-md-sm text-danger text-body-sm"
                                    >
                                        {error}
                                    </motion.div>
                                )}
                            </div>
                        </motion.div>
                    )}
                </AnimatePresence>
            </div>
        </MoviePosterBackground>
    );
};
