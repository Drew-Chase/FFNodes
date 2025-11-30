import {useEffect, useState} from "react";
import {motion} from "framer-motion";
import {invoke} from "@tauri-apps/api/core";
import {listen} from "@tauri-apps/api/event";
import {useNavigate} from "react-router-dom";
import {addToast} from "@heroui/toast";
import {ClientConfig, useConfigStore} from "../stores/useConfigStore";
import {EncodingJob, useJobStore} from "../stores/useJobStore";
import {VideoBackground} from "../components/VideoBackground";
import {CurrentJob} from "../components/CurrentJob";
import {VideoList} from "../components/VideoList";
import {Button} from "../components/ui";
import {BentoGrid, BentoCard, BentoCardHeader, BentoCardContent} from "../components/layout/BentoGrid";
import {Logger} from "../utils/logger";
import {HistoryBento} from "../components/stats/HistoryBento";
import {RemoteUsersBento} from "../components/stats/RemoteUsersBento";
import {LeaderboardBento} from "../components/stats/LeaderboardBento";
import {SettingsModal} from "../components/SettingsModal";
import {ScanProgressBento} from "../components/stats/ScanProgressBento";
import {OverallStatsBento} from "../components/stats/OverallStatsBento";
import {useSystemStatsStore, ScanProgress} from "../stores/useSystemStatsStore";
import {createReconnectingSSE} from "../lib/sse";

export function Dashboard()
{
    const navigate = useNavigate();
    const {config, setConfig} = useConfigStore();
    const {setJobQueue, setCurrentJob, updateProgress, setProcessing} = useJobStore();
    const [isLoading, setIsLoading] = useState(true);
    const [gpuInfo, setGpuInfo] = useState<any>(null);
    const [isPaused, setIsPaused] = useState(true); // Start paused by default
    const [isSettingsOpen, setIsSettingsOpen] = useState(false);
    const {setOverallStats, setSystemStatus, updateScanProgress, addScanFile} = useSystemStatsStore();

    // Create logger for Dashboard
    const logger = new Logger("Dashboard");

    // Load config and GPU info on mount, start job processing
    useEffect(() =>
    {
        const loadInitialData = async () =>
        {
            logger.info("========== Dashboard Initialization ==========");
            try
            {
                // Load saved config
                logger.info("Loading saved configuration...");
                const savedConfig: ClientConfig = await invoke("load_config");
                if (!savedConfig)
                {
                    logger.warn("No configuration found, redirecting to login");
                    navigate("/login");
                    return;
                }
                logger.info("✓ Configuration loaded", {
                    serverUrl: savedConfig.server_url,
                    displayName: savedConfig.display_name,
                    hasAuthToken: !!savedConfig.auth_token
                });
                setConfig(savedConfig);

                // Get GPU info
                logger.info("Getting GPU information...");
                const gpu = await invoke("get_gpu_info");
                logger.info("✓ GPU info retrieved", gpu);
                setGpuInfo(gpu);

                // Load system statistics
                logger.info("Loading system statistics...");
                try
                {
                    const [overallStats, systemStatus] = await Promise.all([
                        invoke("get_overall_stats", {config: savedConfig}),
                        invoke("get_system_status", {config: savedConfig})
                    ]);
                    logger.info("✓ System statistics loaded", {overallStats, systemStatus});
                    setOverallStats(overallStats as any);
                    setSystemStatus(systemStatus as any);
                } catch (statsError)
                {
                    logger.warn("Failed to load system statistics:", statsError);
                    // Don't fail the entire initialization if stats fail to load
                }

                // Load active jobs
                logger.info("Loading active jobs from server...");
                const jobs: EncodingJob[] = await invoke("get_active_jobs", {config: savedConfig});
                logger.info(`✓ Loaded ${jobs?.length || 0} active jobs`);
                setJobQueue(jobs || []);

                // Check job manager state (don't auto-start)
                logger.info("Checking job manager state...");
                const state: any = await invoke("get_job_manager_state");
                logger.info("Job manager state:", state);

                if (state.is_processing)
                {
                    // Already running from a previous session
                    logger.info("Job processing already active from previous session");
                    setProcessing(true);
                    setIsPaused(state.is_paused);

                    addToast({
                        title: "Reconnected",
                        description: state.is_paused ? "Job processing paused. Click Resume to continue." : "Job processing active.",
                        color: "primary"
                    });
                } else
                {
                    // Not running - user must manually start
                    logger.info("Job processing not active. User must click Resume to start.");
                    setProcessing(false);
                    setIsPaused(true);

                    addToast({
                        title: "Connected",
                        description: "Connected to server. Click Resume to start processing jobs.",
                        color: "primary"
                    });
                }

                logger.info("========== Dashboard Initialization Complete ==========");
            } catch (error)
            {
                logger.error("✗ Dashboard initialization failed:", error);
                addToast({
                    title: "Error",
                    description: `Failed to load: ${error}`,
                    color: "danger"
                });
            } finally
            {
                setIsLoading(false);
            }
        };

        loadInitialData();

        // Note: Removed cleanup effect that stopped job processing on unmount
        // Processing now continues even when navigating within the app
        // This allows settings modal to open without stopping encoding
    }, [navigate, setConfig, setJobQueue, setProcessing, setOverallStats, setSystemStatus]);

    // Set up periodic refresh for system statistics
    useEffect(() =>
    {
        if (!config) return;

        const refreshStats = async () =>
        {
            try
            {
                const [overallStats, systemStatus] = await Promise.all([
                    invoke("get_overall_stats", {config}),
                    invoke("get_system_status", {config})
                ]);
                setOverallStats(overallStats as any);
                setSystemStatus(systemStatus as any);
            } catch (error)
            {
                logger.error("Failed to refresh system statistics:", error);
            }
        };

        // Refresh every 30 seconds
        const interval = setInterval(refreshStats, 30000);

        return () => clearInterval(interval);
    }, [config, setOverallStats, setSystemStatus]);

    // Set up SSE connection for scan progress
    useEffect(() => {
        if (!config) return;

        const serverUrl = config.server_url || '';
        const cleanupScanSSE = createReconnectingSSE(
            `${serverUrl}/api/public/monitoring/scan/progress`,
            (data) => {
                // Handle SSE messages
                if (typeof data === 'object' && 'total_files' in data) {
                    const progress = data as ScanProgress;
                    updateScanProgress(progress);

                    // Add file to history if present
                    if (progress.current_file) {
                        addScanFile(progress.current_file);
                    }
                }
            },
            (connected) => {
                logger.info(`Scan progress SSE ${connected ? 'connected' : 'disconnected'}`);
            }
        );

        return () => {
            cleanupScanSSE();
        };
    }, [config, updateScanProgress, addScanFile]);

    // Set up event listeners for job updates
    useEffect(() =>
    {
        const unlistenPromises: Promise<() => void>[] = [];

        // Listen for job started
        unlistenPromises.push(
            listen("job-started", (event: any) =>
            {
                logger.info("📥 Received job-started event", event.payload);
                setCurrentJob(event.payload.job);
                updateProgress({
                    phase: "downloading",
                    totalFrames: event.payload.total_frames || 0
                });
                addToast({
                    title: "Info",
                    description: `Started encoding: ${event.payload.job.media_file_path}`,
                    color: "primary"
                });
            })
        );

        // Listen for frame extracted
        unlistenPromises.push(
            listen("frame-extracted", (event: any) =>
            {
                console.log("Frame extracted");
                updateProgress({extractedFrame: event.payload});
            })
        );

        // Listen for encoding progress
        unlistenPromises.push(
            listen("encoding-progress", (event: any) =>
            {
                const progress = event.payload;
                updateProgress({
                    phase: "encoding",
                    frame: progress.frame,
                    fps: progress.fps,
                    bitrate: progress.bitrate,
                    speed: progress.speed,
                    percentage: progress.percentage
                });
            })
        );

        // Listen for download started
        unlistenPromises.push(
            listen("download-started", (event: any) =>
            {
                logger.info("📥 Download started", event.payload);
                updateProgress({
                    phase: "downloading",
                    totalTransferBytes: event.payload.total_bytes,
                    transferredBytes: 0,
                    transferSpeed: 0,
                    percentage: 0
                });
            })
        );

        // Listen for download progress
        unlistenPromises.push(
            listen("download-progress", (event: any) =>
            {
                const progress = event.payload;
                updateProgress({
                    phase: "downloading",
                    transferredBytes: progress.transferred_bytes,
                    totalTransferBytes: progress.total_bytes,
                    transferSpeed: progress.bytes_per_second,
                    percentage: progress.percentage
                });
            })
        );

        // Listen for download completed
        unlistenPromises.push(
            listen("download-completed", (event: any) =>
            {
                logger.info("✓ Download completed", event.payload);
                // Clear transfer progress, ready for encoding phase
                updateProgress({
                    transferredBytes: undefined,
                    totalTransferBytes: undefined,
                    transferSpeed: undefined
                });
            })
        );

        // Listen for upload started
        unlistenPromises.push(
            listen("upload-started", (event: any) =>
            {
                logger.info("📤 Upload started", event.payload);
                updateProgress({
                    phase: "uploading",
                    totalTransferBytes: event.payload.total_bytes,
                    transferredBytes: 0,
                    transferSpeed: 0,
                    percentage: 0
                });
            })
        );

        // Listen for upload progress
        unlistenPromises.push(
            listen("upload-progress", (event: any) =>
            {
                const progress = event.payload;
                updateProgress({
                    phase: "uploading",
                    transferredBytes: progress.transferred_bytes,
                    totalTransferBytes: progress.total_bytes,
                    transferSpeed: progress.bytes_per_second,
                    percentage: progress.percentage
                });
            })
        );

        // Listen for upload completed
        unlistenPromises.push(
            listen("upload-completed", (event: any) =>
            {
                logger.info("✓ Upload completed", event.payload);
                // Keep stats at 100% for 2 seconds before transitioning
                setTimeout(() =>
                {
                    updateProgress({phase: "completing"});
                }, 2000);
            })
        );

        // Listen for job completed
        unlistenPromises.push(
            listen("job-completed", (event: any) =>
            {
                console.log("Job completed:", event.payload);
                addToast({
                    title: "Success",
                    description: "Job completed successfully!",
                    color: "success"
                });
                setCurrentJob(null);
                updateProgress(null);
            })
        );

        // Listen for job errors
        unlistenPromises.push(
            listen("job-error", (event: any) =>
            {
                console.error("Job error:", event.payload);
                addToast({
                    title: "Error",
                    description: `Error: ${event.payload}`,
                    color: "danger",
                    timeout: 6000
                });
            })
        );

        // Cleanup listeners on unmount
        return () =>
        {
            Promise.all(unlistenPromises).then((unlisteners) =>
            {
                unlisteners.forEach((unlisten) => unlisten());
            });
        };
    }, [setCurrentJob, updateProgress]);

    // Handle pause/resume
    const handlePauseResume = async () =>
    {
        try
        {
            if (isPaused)
            {
                // Check if processing is actually running
                const state: any = await invoke("get_job_manager_state");

                if (!state.is_processing)
                {
                    // Start processing for the first time
                    logger.info("Starting job processing for the first time...");
                    await invoke("start_job_processing", {config, gpu: gpuInfo});
                    setProcessing(true);
                    setIsPaused(false);
                    addToast({
                        title: "Started",
                        description: "Job processing started",
                        color: "success"
                    });
                } else
                {
                    // Resume existing processing
                    await invoke("resume_job_processing");
                    setIsPaused(false);
                    addToast({
                        title: "Resumed",
                        description: "Job processing resumed",
                        color: "success"
                    });
                }
            } else
            {
                await invoke("pause_job_processing");
                setIsPaused(true);
                addToast({
                    title: "Paused",
                    description: "Job processing paused",
                    color: "primary"
                });
            }
        } catch (error)
        {
            addToast({
                title: "Error",
                description: `Failed to ${isPaused ? "resume" : "pause"}: ${error}`,
                color: "danger"
            });
        }
    };

    // Handle cancel current job
    const handleCancel = async () =>
    {
        try
        {
            logger.info("Cancelling current job...");
            await invoke("stop_job_processing");
            setCurrentJob(null);
            updateProgress(null);

            // Also pause job processing
            await invoke("pause_job_processing");
            setIsPaused(true);
            setProcessing(false);

            addToast({
                title: "Job Cancelled",
                description: "Current job has been cancelled and job processing paused",
                color: "warning"
            });
        } catch (error)
        {
            addToast({
                title: "Error",
                description: `Failed to cancel: ${error}`,
                color: "danger"
            });
        }
    };

    if (isLoading)
    {
        return (
            <div className="min-h-screen flex items-center justify-center">
                <div className="text-center">
                    <div className="neon-spinner mx-auto mb-4"/>
                    <p className="text-lg">Loading Dashboard...</p>
                </div>
            </div>
        );
    }

    return (
        <>
            {/* Dynamic Video Background */}
            <VideoBackground/>

            {/* Main Content */}
            <div className="min-h-screen p-6 md:p-8 relative z-10 max-h-screen overflow-y-auto bg-base-200 dark:bg-base-100">
                {/* Header */}
                <motion.header
                    className="mb-6 mt-8"
                    initial={{opacity: 0, y: -20}}
                    animate={{opacity: 1, y: 0}}
                    transition={{duration: 0.3}}
                >
                    <BentoCard elevation={3} background="glass" className="!p-4">
                        <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
                            <div>
                                <h1 className="text-headline-md font-medium text-primary mb-1">
                                    FFNodes Client
                                </h1>
                                <p className="text-body-md text-foreground/70">
                                    {config?.display_name || "Encoding Client"} • {gpuInfo?.vendor || "Unknown GPU"}
                                </p>
                            </div>

                            <div className="flex gap-2 flex-wrap">
                                <Button
                                    color={isPaused ? "success" : "primary"}
                                    onPress={handlePauseResume}
                                    className="rounded-md-lg shadow-md-2"
                                >
                                    <iconify-icon icon={isPaused ? "mdi:play" : "mdi:pause"} class="text-lg mr-1"/>
                                    {isPaused ? "Resume" : "Pause"}
                                </Button>
                                <Button
                                    color="danger"
                                    onPress={handleCancel}
                                    className="rounded-md-lg shadow-md-2"
                                >
                                    <iconify-icon icon="mdi:stop" class="text-lg mr-1"/>
                                    Cancel Job
                                </Button>
                                <Button
                                    color="secondary"
                                    onPress={() => setIsSettingsOpen(true)}
                                    className="rounded-md-lg shadow-md-2"
                                >
                                    <iconify-icon icon="mdi:cog" class="text-lg mr-1"/>
                                    Settings
                                </Button>
                            </div>
                        </div>
                    </BentoCard>
                </motion.header>

                {/* Scan Progress Indicator - shown only when scanning */}
                <ScanProgressBento />

                {/* Bento Grid Layout */}
                <BentoGrid columns={6} gap="md">
                    {/* Overall System Statistics - Full Width */}
                    <OverallStatsBento/>

                    {/* Current Job - Large card spanning 4 columns and 2 rows */}
                    <BentoCard
                        colSpan={4}
                        rowSpan={2}
                        elevation={4}
                        background="glass"
                        className="min-h-[400px]"
                    >
                        <CurrentJob/>
                    </BentoCard>

                    {/* Video Queue - Tall card on the right */}
                    <BentoCard
                        colSpan={2}
                        rowSpan={2}
                        elevation={3}
                        background="glass"
                        className="min-h-[400px]"
                    >
                        <VideoList/>
                    </BentoCard>

                    {/* History Bento Box */}
                    <HistoryBento/>

                    {/* Remote Users Bento Box */}
                    <RemoteUsersBento/>

                    {/* Leaderboard Bento Box */}
                    <LeaderboardBento/>

                    {/* Connection Status Card */}
                    <BentoCard
                        colSpan={3}
                        elevation={2}
                        background="gradient"
                    >
                        <BentoCardHeader
                            title="Connection Status"
                            icon={<iconify-icon icon="mdi:lan-connect" class="text-2xl"/>}
                        />
                        <BentoCardContent>
                            <div className="flex items-center gap-3 mt-2">
                                <div className="w-3 h-3 rounded-full bg-success animate-pulse shadow-md-2 shadow-success"/>
                                <div>
                                    <p className="text-body-lg font-medium text-foreground">
                                        Connected
                                    </p>
                                    <p className="text-body-sm text-foreground/70">
                                        {config?.server_url || "Server"}
                                    </p>
                                </div>
                            </div>
                        </BentoCardContent>
                    </BentoCard>

                    {/* GPU Info Card */}
                    <BentoCard
                        colSpan={3}
                        elevation={2}
                        background="gradient"
                    >
                        <BentoCardHeader
                            title="GPU Information"
                            icon={<iconify-icon icon="mdi:memory" class="text-2xl"/>}
                        />
                        <BentoCardContent>
                            <div className="mt-2 space-y-1">
                                <p className="text-body-lg font-medium text-foreground">
                                    {gpuInfo?.name || "Unknown GPU"}
                                </p>
                                <p className="text-body-sm text-foreground/70">
                                    {gpuInfo?.vendor || "Unknown Vendor"}
                                </p>
                                {gpuInfo?.memory && (
                                    <p className="text-body-sm text-foreground/60">
                                        {gpuInfo.memory} MB VRAM
                                    </p>
                                )}
                            </div>
                        </BentoCardContent>
                    </BentoCard>

                    {/* Quick Actions Card */}
                    <BentoCard
                        colSpan={2}
                        elevation={2}
                        background="solid"
                        hover
                        onClick={() => setIsSettingsOpen(true)}
                    >
                        <BentoCardHeader
                            title="Quick Settings"
                            icon={<iconify-icon icon="mdi:settings-outline" class="text-2xl"/>}
                        />
                        <BentoCardContent>
                            <p className="text-body-sm text-foreground/70 mt-2">
                                Update server connection and preferences
                            </p>
                        </BentoCardContent>
                    </BentoCard>

                    {/* Stats Card - Jobs Processed */}
                    <BentoCard
                        colSpan={2}
                        elevation={2}
                        background="gradient"
                    >
                        <BentoCardHeader
                            title="Jobs Today"
                            icon={<iconify-icon icon="mdi:chart-line" class="text-2xl"/>}
                        />
                        <BentoCardContent>
                            <div className="mt-2">
                                <p className="text-display-sm font-normal text-primary">
                                    {useJobStore.getState().jobQueue.length}
                                </p>
                                <p className="text-body-sm text-foreground/70">
                                    Active in queue
                                </p>
                            </div>
                        </BentoCardContent>
                    </BentoCard>

                    {/* Processing Status */}
                    <BentoCard
                        colSpan={2}
                        elevation={2}
                        background="gradient"
                    >
                        <BentoCardHeader
                            title="Status"
                            icon={<iconify-icon icon="mdi:state-machine" class="text-2xl"/>}
                        />
                        <BentoCardContent>
                            <div className="mt-2">
                                <p className="text-title-lg font-medium text-foreground">
                                    {isPaused ? "Paused" : "Processing"}
                                </p>
                                <p className="text-body-sm text-foreground/70">
                                    {isPaused ? "Resume to continue" : "Actively encoding"}
                                </p>
                            </div>
                        </BentoCardContent>
                    </BentoCard>
                </BentoGrid>
            </div>

            {/* Settings Modal */}
            <SettingsModal
                isOpen={isSettingsOpen}
                onClose={() => setIsSettingsOpen(false)}
            />
        </>
    );
}
