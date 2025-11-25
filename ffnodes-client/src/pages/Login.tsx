import React, {useState} from "react";
import {useNavigate} from "react-router-dom";
import {AnimatePresence, motion} from "framer-motion";
import {Button, Input} from "../components/ui";
import {MoviePosterBackground} from "../components/MoviePosterScroll";
import {OAuthService} from "../services/oauth";
import {QRCodeSVG} from "qrcode.react";

export const Login: React.FC = () =>
{
    const navigate = useNavigate();
    const [email, setEmail] = useState("");
    const [displayName, setDisplayName] = useState("");
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState("");
    const [showQR, setShowQR] = useState(false);
    const [showDisplayNameLogin, setShowDisplayNameLogin] = useState(false);

    const handleOAuthLogin = async (provider: "google" | "github" | "microsoft" | "facebook") =>
    {
        setLoading(true);
        setError("");

        try
        {
            await OAuthService.login(provider);
            navigate("/dashboard");
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

        setLoading(true);
        setError("");

        // TODO: Implement email login with magic link or password
        // For now, show error
        setError("Email login not yet implemented");
        setLoading(false);
    };

    const handleDisplayNameLogin = () =>
    {
        if (!displayName.trim())
        {
            setError("Please enter a display name");
            return;
        }

        OAuthService.loginWithDisplayName(displayName.trim());
        navigate("/setup"); // Go to setup to configure server connection
    };

    // Generate QR code URL for mobile login
    const qrCodeUrl = typeof window !== "undefined" ? window.location.href : "";

    return (
        <MoviePosterBackground>
            <div className="flex items-center justify-center min-h-screen p-4">
                <AnimatePresence mode="wait">
                    {!showDisplayNameLogin ? (
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
                                    onClick={() => setShowDisplayNameLogin(true)}
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

                            {/* Matter branding (optional) */}
                            <div className="mt-4 text-center text-body-sm text-white/70">
                                <span>Powered by FFNodes</span>
                            </div>
                        </motion.div>
                    ) : (
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
                                <button
                                    onClick={() => setShowDisplayNameLogin(false)}
                                    className="mb-4 text-foreground/60 hover:text-foreground transition-colors flex items-center gap-2"
                                >
                                    <iconify-icon icon="mdi:arrow-left" class="text-xl"/>
                                    <span className="text-body-md">Back to OAuth</span>
                                </button>

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
                                        onPress={handleDisplayNameLogin}
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
                            </div>
                        </motion.div>
                    )}
                </AnimatePresence>
            </div>
        </MoviePosterBackground>
    );
};
