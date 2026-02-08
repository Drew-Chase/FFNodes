import { create } from 'zustand';

export interface ClientConfig {
  server_url: string;
  server_guid: string;
  display_name: string;
  computer_name: string;
  client_id: string | null;
  auth_token: string | null;
  ffmpeg_template: string | null;
  auto_start_processing?: boolean;
  skip_if_output_larger?: boolean;
  output_size_margin_percent?: number;
}

interface ConfigStore {
  config: ClientConfig | null;
  isLoading: boolean;
  setConfig: (config: ClientConfig) => void;
  clearConfig: () => void;
  updateConfig: (updates: Partial<ClientConfig>) => void;
}

export const useConfigStore = create<ConfigStore>((set) => ({
  config: null,
  isLoading: false,
  setConfig: (config) => set({ config }),
  clearConfig: () => set({ config: null }),
  updateConfig: (updates) =>
    set((state) => ({
      config: state.config ? { ...state.config, ...updates } : null,
    })),
}));
