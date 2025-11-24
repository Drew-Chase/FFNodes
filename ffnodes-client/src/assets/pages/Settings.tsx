import { useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';
import { useNavigate } from 'react-router-dom';
import toast, { Toaster } from 'react-hot-toast';
import { useConfigStore } from '../../stores/useConfigStore';
import { NeonCard } from '../components/NeonCard';
import { NeonButton } from '../components/NeonButton';
import { NeonInput } from '../components/NeonInput';
import { GlowText } from '../components/GlowText';

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
  const [formData, setFormData] = useState<FormData>({
    serverUrl: config?.server_url || '',
    serverGuid: config?.server_guid || '',
    displayName: config?.display_name || '',
  });
  const [gpuInfo, setGpuInfo] = useState<GpuInfo | null>(null);
  const [isTesting, setIsTesting] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState<'idle' | 'success' | 'error'>('idle');

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

  const handleInputChange = (field: keyof FormData) => (value: string) => {
    setFormData((prev) => ({ ...prev, [field]: value }));
    setConnectionStatus('idle'); // Reset status on change
  };

  const handleTestConnection = async () => {
    if (!formData.serverUrl || !formData.serverGuid || !formData.displayName) {
      toast.error('All fields are required', {
        className: 'toast-neon error',
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
      toast.success('Connection successful!', {
        className: 'toast-neon success',
      });
    } catch (error) {
      setConnectionStatus('error');
      toast.error(`Connection failed: ${error}`, {
        className: 'toast-neon error',
        duration: 6000,
      });
    } finally {
      setIsTesting(false);
    }
  };

  const handleSave = async () => {
    if (!formData.serverUrl || !formData.serverGuid || !formData.displayName) {
      toast.error('All fields are required', {
        className: 'toast-neon error',
      });
      return;
    }

    setIsSaving(true);

    try {
      const newConfig = await invoke('test_connection', {
        input: {
          server_url: formData.serverUrl,
          server_guid: formData.serverGuid,
          display_name: formData.displayName,
        },
      });

      setConfig(newConfig);
      toast.success('Settings saved successfully!', {
        className: 'toast-neon success',
      });

      // Wait a bit before navigating
      setTimeout(() => {
        navigate('/dashboard');
      }, 1000);
    } catch (error) {
      toast.error(`Failed to save: ${error}`, {
        className: 'toast-neon error',
        duration: 6000,
      });
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <>
      <Toaster
        position="top-right"
        toastOptions={{
          duration: 4000,
          style: {
            background: 'transparent',
            boxShadow: 'none',
          },
        }}
      />

      <div className="min-h-screen p-8 bg-[var(--bg-dark)]">
        <div className="max-w-3xl mx-auto">
          {/* Header */}
          <motion.div
            className="mb-8"
            initial={{ opacity: 0, y: -20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
          >
            <GlowText size="2xl" className="mb-2">
              Settings
            </GlowText>
            <p className="text-gray-400">Configure your FFNodes client</p>
          </motion.div>

          <div className="space-y-6">
            {/* Server Configuration */}
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5, delay: 0.1 }}
            >
              <NeonCard variant="glass">
                <div className="space-y-4">
                  <GlowText variant="accent" className="mb-4">
                    Server Configuration
                  </GlowText>

                  <NeonInput
                    label="Server URL"
                    placeholder="http://localhost:8080"
                    value={formData.serverUrl}
                    onChange={handleInputChange('serverUrl')}
                    required
                  />

                  <NeonInput
                    label="Server GUID"
                    placeholder="Enter server secret code"
                    value={formData.serverGuid}
                    onChange={handleInputChange('serverGuid')}
                    required
                  />

                  <NeonInput
                    label="Display Name"
                    placeholder="My Encoding Client"
                    value={formData.displayName}
                    onChange={handleInputChange('displayName')}
                    required
                  />

                  <div className="flex gap-3 mt-6">
                    <NeonButton
                      onClick={handleTestConnection}
                      loading={isTesting}
                      disabled={isTesting || !formData.serverUrl || !formData.serverGuid || !formData.displayName}
                      variant="accent"
                    >
                      Test Connection
                    </NeonButton>

                    {connectionStatus === 'success' && (
                      <motion.div
                        className="flex items-center gap-2"
                        initial={{ opacity: 0, scale: 0.8 }}
                        animate={{ opacity: 1, scale: 1 }}
                      >
                        <div className="w-2 h-2 rounded-full bg-[var(--neon-green)]" />
                        <span className="text-[var(--neon-green)] text-sm">Connected</span>
                      </motion.div>
                    )}

                    {connectionStatus === 'error' && (
                      <motion.div
                        className="flex items-center gap-2"
                        initial={{ opacity: 0, scale: 0.8 }}
                        animate={{ opacity: 1, scale: 1 }}
                      >
                        <div className="w-2 h-2 rounded-full bg-[var(--neon-primary)]" />
                        <span className="text-[var(--neon-primary)] text-sm">Failed</span>
                      </motion.div>
                    )}
                  </div>
                </div>
              </NeonCard>
            </motion.div>

            {/* GPU Information */}
            {gpuInfo && (
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.5, delay: 0.2 }}
              >
                <NeonCard variant="glass">
                  <div className="space-y-4">
                    <GlowText variant="purple" className="mb-4">
                      GPU Information
                    </GlowText>

                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <p className="text-gray-400 text-sm mb-1">Vendor</p>
                        <p className="text-white font-medium">{gpuInfo.vendor}</p>
                      </div>
                      <div>
                        <p className="text-gray-400 text-sm mb-1">GPU Model</p>
                        <p className="text-white font-medium">{gpuInfo.name}</p>
                      </div>
                      <div>
                        <p className="text-gray-400 text-sm mb-1">H.264 Encoder</p>
                        <p className="text-[var(--neon-accent)] font-mono text-sm">{gpuInfo.encoder_h264}</p>
                      </div>
                      <div>
                        <p className="text-gray-400 text-sm mb-1">H.265 Encoder</p>
                        <p className="text-[var(--neon-accent)] font-mono text-sm">{gpuInfo.encoder_h265}</p>
                      </div>
                    </div>
                  </div>
                </NeonCard>
              </motion.div>
            )}

            {/* Current Configuration */}
            {config && (
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.5, delay: 0.3 }}
              >
                <NeonCard variant="glass">
                  <div className="space-y-3">
                    <GlowText variant="primary" className="mb-4">
                      Current Session
                    </GlowText>

                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-gray-400">Client ID:</span>
                        <span className="text-white font-mono">{config.client_id || 'Not set'}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-400">Computer Name:</span>
                        <span className="text-white">{config.computer_name}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-400">Auth Status:</span>
                        <span className={config.auth_token ? 'text-[var(--neon-green)]' : 'text-gray-400'}>
                          {config.auth_token ? 'Authenticated' : 'Not authenticated'}
                        </span>
                      </div>
                    </div>
                  </div>
                </NeonCard>
              </motion.div>
            )}

            {/* Action Buttons */}
            <motion.div
              className="flex gap-4 justify-end"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ duration: 0.5, delay: 0.4 }}
            >
              <NeonButton variant="accent" onClick={() => navigate('/dashboard')}>
                Cancel
              </NeonButton>
              <NeonButton
                variant="primary"
                onClick={handleSave}
                loading={isSaving}
                disabled={isSaving || !formData.serverUrl || !formData.serverGuid || !formData.displayName}
              >
                Save & Apply
              </NeonButton>
            </motion.div>
          </div>
        </div>
      </div>
    </>
  );
}
