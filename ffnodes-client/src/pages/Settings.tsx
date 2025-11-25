import { useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';
import { useNavigate } from 'react-router-dom';
import { addToast } from '@heroui/toast';
import {ClientConfig, useConfigStore} from "../stores/useConfigStore";
import { Button, Input, Card } from '../components/ui';

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

      setConfig(newConfig);
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

  return (
      <div className="min-h-screen p-8">
        <div className="max-w-3xl mx-auto">
          {/* Header */}
          <motion.div
            className="mb-8"
            initial={{ opacity: 0, y: -20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
          >
            <h1 className="text-2xl font-bold text-primary mb-2">
              Settings
            </h1>
            <p className="text-default-500">Configure your FFNodes client</p>
          </motion.div>

          <div className="space-y-6">
            {/* Server Configuration */}
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5, delay: 0.1 }}
            >
              <Card variant="glass">
                <div className="space-y-4">
                  <h2 className="text-lg font-semibold text-secondary mb-4">
                    Server Configuration
                  </h2>

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

                  <div className="flex gap-3 mt-6">
                    <Button
                      onClick={handleTestConnection}
                      loading={isTesting}
                      isDisabled={isTesting || !formData.serverUrl || !formData.serverGuid || !formData.displayName}
                      variant="accent"
                    >
                      Test Connection
                    </Button>

                    {connectionStatus === 'success' && (
                      <div className="flex items-center gap-2">
                        <div className="w-2 h-2 rounded-full bg-success" />
                        <span className="text-success text-sm">Connected</span>
                      </div>
                    )}

                    {connectionStatus === 'error' && (
                      <div className="flex items-center gap-2">
                        <div className="w-2 h-2 rounded-full bg-danger" />
                        <span className="text-danger text-sm">Failed</span>
                      </div>
                    )}
                  </div>
                </div>
              </Card>
            </motion.div>

            {/* GPU Information */}
            {gpuInfo && (
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.5, delay: 0.2 }}
              >
                <Card variant="glass">
                  <div className="space-y-4">
                    <h2 className="text-lg font-semibold text-purple-500 mb-4">
                      GPU Information
                    </h2>

                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <p className="text-default-500 text-sm mb-1">Vendor</p>
                        <p className="text-foreground font-medium">{gpuInfo.vendor}</p>
                      </div>
                      <div>
                        <p className="text-default-500 text-sm mb-1">GPU Model</p>
                        <p className="text-foreground font-medium">{gpuInfo.name}</p>
                      </div>
                      <div>
                        <p className="text-default-500 text-sm mb-1">H.264 Encoder</p>
                        <p className="text-secondary font-mono text-sm">{gpuInfo.encoder_h264}</p>
                      </div>
                      <div>
                        <p className="text-default-500 text-sm mb-1">H.265 Encoder</p>
                        <p className="text-secondary font-mono text-sm">{gpuInfo.encoder_h265}</p>
                      </div>
                    </div>
                  </div>
                </Card>
              </motion.div>
            )}

            {/* Current Configuration */}
            {config && (
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.5, delay: 0.3 }}
              >
                <Card variant="glass">
                  <div className="space-y-3">
                    <h2 className="text-lg font-semibold text-primary mb-4">
                      Current Session
                    </h2>

                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-default-500">Client ID:</span>
                        <span className="text-foreground font-mono">{config.client_id || 'Not set'}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-default-500">Computer Name:</span>
                        <span className="text-foreground">{config.computer_name}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-default-500">Auth Status:</span>
                        <span className={config.auth_token ? 'text-success' : 'text-default-500'}>
                          {config.auth_token ? 'Authenticated' : 'Not authenticated'}
                        </span>
                      </div>
                    </div>
                  </div>
                </Card>
              </motion.div>
            )}

            {/* Action Buttons */}
            <motion.div
              className="flex gap-4 justify-end"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ duration: 0.5, delay: 0.4 }}
            >
              <Button variant="accent" onClick={() => navigate('/dashboard')}>
                Cancel
              </Button>
              <Button
                variant="primary"
                onClick={handleSave}
                loading={isSaving}
                isDisabled={isSaving || !formData.serverUrl || !formData.serverGuid || !formData.displayName}
              >
                Save & Apply
              </Button>
            </motion.div>
          </div>
        </div>
      </div>
  );
}
