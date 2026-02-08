import { useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';
import { useNavigate } from 'react-router-dom';
import { addToast } from '@heroui/toast';
import {ClientConfig, useConfigStore} from "../stores/useConfigStore";
import { Button, Input } from '../components/ui';
import { BentoGrid, BentoCard, BentoCardHeader, BentoCardContent, BentoCardFooter } from '../components/layout/BentoGrid';
import { useAuthStore } from '../stores/useAuthStore';
import { OAuthService } from '../services/oauth';
import { useThemeStore } from '../stores/useThemeStore';

interface FormData {
  serverUrl: string;
  serverGuid: string;
  displayName: string;
}

interface GpuInfo {
  vendor: string;
  name: string;
  encoder_h264: string;
  encoder_h265: string;
}

export function Settings() {
  const navigate = useNavigate();
  const { config, setConfig } = useConfigStore();
  const { theme, setTheme } = useThemeStore();
  const [formData, setFormData] = useState<FormData>({
    serverUrl: config?.server_url || '',
    serverGuid: config?.server_guid || '',
    displayName: config?.display_name || '',
  });
  const [gpuInfo, setGpuInfo] = useState<GpuInfo | null>(null);
  const [isTesting, setIsTesting] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState<'idle' | 'success' | 'error'>('idle');
  const [ffmpegCommand, setFfmpegCommand] = useState<string | null>(null);
  const [skipIfOutputLarger, setSkipIfOutputLarger] = useState(
    config?.skip_if_output_larger ?? false
  );
  const [outputSizeMargin, setOutputSizeMargin] = useState(
    config?.output_size_margin_percent ?? 1.0
  );

  useEffect(() => {
    // Load GPU info
    const loadGpuInfo = async () => {
      try {
        const gpu = await invoke<GpuInfo>('get_gpu_info');
        setGpuInfo(gpu);
      } catch (error) {
        console.error('Failed to load GPU info:', error);
      }
    };
    loadGpuInfo();
  }, []);

  useEffect(() => {
    // Load FFmpeg command when config and GPU info are available
    const loadFfmpegCommand = async () => {
      if (config && gpuInfo && config.ffmpeg_template) {
        try {
          const command = await invoke<string>('get_ffmpeg_command', {
            config,
            gpu: gpuInfo,
          });
          setFfmpegCommand(command);
        } catch (error) {
          console.error('Failed to load FFmpeg command:', error);
        }
      }
    };
    loadFfmpegCommand();
  }, [config, gpuInfo]);

  useEffect(() => {
    // Sync size monitoring settings when config changes
    if (config) {
      setSkipIfOutputLarger(config.skip_if_output_larger ?? false);
      setOutputSizeMargin(config.output_size_margin_percent ?? 1.0);
    }
  }, [config]);

  const handleInputChange = (field: keyof FormData) => (value: string) => {
    setFormData((prev) => ({ ...prev, [field]: value }));
    setConnectionStatus('idle'); // Reset status on change
  };

  const handleTestConnection = async () => {
    if (!formData.serverUrl || !formData.serverGuid || !formData.displayName) {
      addToast({
        title: 'Error',
        description: 'All fields are required',
        color: 'danger'
      });
      return;
    }

    setIsTesting(true);
    setConnectionStatus('idle');

    try {
      await invoke('test_connection', {
        input: {
          server_url: formData.serverUrl,
          server_guid: formData.serverGuid,
          display_name: formData.displayName,
        },
      });

      setConnectionStatus('success');
      addToast({
        title: 'Success',
        description: 'Connection successful!',
        color: 'success'
      });
    } catch (error) {
      setConnectionStatus('error');
      addToast({
        title: 'Error',
        description: `Connection failed: ${error}`,
        color: 'danger',
        timeout: 6000
      });
    } finally {
      setIsTesting(false);
    }
  };

  const handleSave = async () => {
    if (!formData.serverUrl || !formData.serverGuid || !formData.displayName) {
      addToast({
        title: 'Error',
        description: 'All fields are required',
        color: 'danger'
      });
      return;
    }

    setIsSaving(true);

    try {
      const newConfig: ClientConfig = await invoke('test_connection', {
        input: {
          server_url: formData.serverUrl,
          server_guid: formData.serverGuid,
          display_name: formData.displayName,
        },
      });

      // Add size monitoring settings
      const updatedConfig = {
        ...newConfig,
        skip_if_output_larger: skipIfOutputLarger,
        output_size_margin_percent: outputSizeMargin,
      };

      // Save the updated config
      await invoke('save_config', { config: updatedConfig });

      setConfig(updatedConfig);
      addToast({
        title: 'Success',
        description: 'Settings saved successfully!',
        color: 'success'
      });

      // Wait a bit before navigating
      setTimeout(() => {
        navigate('/dashboard');
      }, 1000);
    } catch (error) {
      addToast({
        title: 'Error',
        description: `Failed to save: ${error}`,
        color: 'danger',
        timeout: 6000
      });
    } finally {
      setIsSaving(false);
    }
  };

  const { user, isAuthenticated } = useAuthStore();

  const handleLogout = () => {
    OAuthService.logout();
    navigate('/login');
  };

  return (
      <div className="min-h-screen p-6 md:p-8">
        {/* Header */}
        <motion.header
          className="mb-6 mt-8 max-w-7xl mx-auto"
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.3 }}
        >
          <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
            <div>
              <h1 className="text-headline-lg font-medium text-primary mb-1">
                Settings
              </h1>
              <p className="text-body-md text-foreground/70">Configure your FFNodes client</p>
            </div>
            <Button
              color="secondary"
              onClick={() => navigate('/dashboard')}
              className="rounded-md-lg shadow-md-2"
            >
              <iconify-icon icon="mdi:arrow-left" class="text-lg mr-1" />
              Back to Dashboard
            </Button>
          </div>
        </motion.header>

        <div className="max-w-7xl mx-auto">
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
                icon={<iconify-icon icon="mdi:server" class="text-2xl" />}
              />
              <BentoCardContent>
                <div className="space-y-4 mt-4">
                  <Input
                    label="Server URL"
                    placeholder="http://localhost:8080"
                    value={formData.serverUrl}
                    onChange={handleInputChange('serverUrl')}
                    required
                  />

                  <Input
                    label="Server GUID"
                    placeholder="Enter server secret code"
                    value={formData.serverGuid}
                    onChange={handleInputChange('serverGuid')}
                    required
                  />

                  <Input
                    label="Display Name"
                    placeholder="My Encoding Client"
                    value={formData.displayName}
                    onChange={handleInputChange('displayName')}
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

                  {connectionStatus === 'success' && (
                    <div className="flex items-center gap-2">
                      <div className="w-2 h-2 rounded-full bg-success animate-pulse" />
                      <span className="text-success text-body-sm font-medium">Connected</span>
                    </div>
                  )}

                  {connectionStatus === 'error' && (
                    <div className="flex items-center gap-2">
                      <div className="w-2 h-2 rounded-full bg-danger" />
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
                subtitle={isAuthenticated ? 'Logged in' : 'Not logged in'}
                icon={<iconify-icon icon="mdi:account-circle" class="text-2xl" />}
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
                    onClick={handleLogout}
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
                icon={<iconify-icon icon="mdi:chip" class="text-2xl" />}
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
                icon={<iconify-icon icon="mdi:information-outline" class="text-2xl" />}
              />
              <BentoCardContent>
                {config ? (
                  <div className="mt-4 space-y-3">
                    <div className="flex justify-between items-center">
                      <span className="text-body-sm text-foreground/60">Client ID</span>
                      <span className="text-body-sm font-mono text-foreground">{config.client_id || 'Not set'}</span>
                    </div>
                    <div className="flex justify-between items-center">
                      <span className="text-body-sm text-foreground/60">Computer Name</span>
                      <span className="text-body-sm text-foreground">{config.computer_name}</span>
                    </div>
                    <div className="flex justify-between items-center">
                      <span className="text-body-sm text-foreground/60">Auth Status</span>
                      <span className={`text-body-sm font-medium ${config.auth_token ? 'text-success' : 'text-foreground/60'}`}>
                        {config.auth_token ? 'Authenticated' : 'Not authenticated'}
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

            {/* Encoding Settings */}
            <BentoCard
              colSpan={4}
              elevation={2}
              background="glass"
            >
              <BentoCardHeader
                title="Encoding Settings"
                subtitle="Configure encoding behavior"
                icon={<iconify-icon icon="mdi:cog" class="text-2xl" />}
              />
              <BentoCardContent>
                <div className="mt-4 space-y-4">
                  <div className="flex items-start gap-3">
                    <input
                      type="checkbox"
                      id="skip-if-larger"
                      checked={skipIfOutputLarger}
                      onChange={(e) => setSkipIfOutputLarger(e.target.checked)}
                      className="mt-1 w-4 h-4 rounded border-foreground/30 bg-content2 text-primary focus:ring-2 focus:ring-primary"
                    />
                    <div className="flex-1">
                      <label htmlFor="skip-if-larger" className="text-body-md font-medium text-foreground cursor-pointer">
                        Skip encoding if output file becomes larger than input
                      </label>
                      <p className="text-body-sm text-foreground/60 mt-1">
                        Automatically abort encoding if the output file size exceeds the input file size. This prevents wasting resources on counterproductive encodes.
                      </p>
                    </div>
                  </div>

                  {skipIfOutputLarger && (
                    <div className="ml-7 pl-4 border-l-2 border-primary/30">
                      <label htmlFor="size-margin" className="text-body-sm text-foreground/70 block mb-2">
                        Safety Margin (%)
                      </label>
                      <div className="flex items-center gap-3">
                        <input
                          type="number"
                          id="size-margin"
                          min="0"
                          max="100"
                          step="0.1"
                          value={outputSizeMargin}
                          onChange={(e) => setOutputSizeMargin(parseFloat(e.target.value) || 0)}
                          className="w-24 px-3 py-2 rounded-md-sm bg-content2 border border-foreground/20 text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
                        />
                        <span className="text-body-sm text-foreground/60">%</span>
                        <span className="text-body-xs text-foreground/50">
                          Allows output to be up to {outputSizeMargin}% larger before aborting
                        </span>
                      </div>
                    </div>
                  )}
                </div>
              </BentoCardContent>
            </BentoCard>

            {/* Theme Settings */}
            <BentoCard
              colSpan={2}
              elevation={2}
              background="gradient"
              hover
            >
              <BentoCardHeader
                title="Appearance"
                subtitle="Theme preferences"
                icon={<iconify-icon icon="mdi:palette" class="text-2xl" />}
              />
              <BentoCardContent>
                <div className="mt-4 flex gap-2">
                  <button
                    onClick={() => setTheme('light')}
                    className={`flex-1 p-3 rounded-md-sm transition-colors ${
                      theme === 'light'
                        ? 'bg-primary text-primary-foreground'
                        : 'bg-content2 hover:bg-content3'
                    }`}
                  >
                    <iconify-icon icon="mdi:white-balance-sunny" class="text-xl" />
                    <p className="text-body-sm mt-1">Light</p>
                  </button>
                  <button
                    onClick={() => setTheme('dark')}
                    className={`flex-1 p-3 rounded-md-sm transition-colors ${
                      theme === 'dark'
                        ? 'bg-primary text-primary-foreground'
                        : 'bg-content2 hover:bg-content3'
                    }`}
                  >
                    <iconify-icon icon="mdi:moon-waning-crescent" class="text-xl" />
                    <p className="text-body-sm mt-1">Dark</p>
                  </button>
                </div>
              </BentoCardContent>
            </BentoCard>

            {/* About */}
            <BentoCard
              colSpan={2}
              elevation={2}
              background="solid"
            >
              <BentoCardHeader
                title="About"
                icon={<iconify-icon icon="mdi:information" class="text-2xl" />}
              />
              <BentoCardContent>
                <div className="mt-4 space-y-2">
                  <p className="text-body-sm text-foreground/70">FFNodes Client</p>
                  <p className="text-body-sm text-foreground/60">Version 0.1.0</p>
                  <p className="text-body-sm text-foreground/50">Built with Tauri & React</p>
                </div>
              </BentoCardContent>
            </BentoCard>

            {/* Actions */}
            <BentoCard
              colSpan={2}
              elevation={2}
              background="gradient"
            >
              <BentoCardContent>
                <Button
                  onClick={handleSave}
                  loading={isSaving}
                  isDisabled={isSaving || !formData.serverUrl || !formData.serverGuid || !formData.displayName}
                  color="primary"
                  className="w-full rounded-md-lg shadow-md-3"
                  size="lg"
                >
                  <iconify-icon icon="mdi:content-save" class="text-xl mr-2" />
                  Save & Apply Changes
                </Button>
              </BentoCardContent>
            </BentoCard>
          </BentoGrid>
        </div>
      </div>
  );
}
