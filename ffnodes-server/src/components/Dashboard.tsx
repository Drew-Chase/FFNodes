import { useEffect } from 'react';
import { motion } from 'framer-motion';
import { BentoGrid, BentoCard, BentoCardHeader, BentoCardContent } from './layout/BentoGrid';
import { OverallStatsBento } from './stats/OverallStatsBento';
import { CurrentJobsBento } from './stats/CurrentJobsBento';
import { ConnectedClientsBento } from './stats/ConnectedClientsBento';
import { LeaderboardBento } from './stats/LeaderboardBento';
import { useDashboardStore } from '../stores/useDashboardStore';

export function Dashboard() {
  const { fetchAllData, systemStatus } = useDashboardStore();

  useEffect(() => {
    // Initial data fetch
    fetchAllData();
  }, [fetchAllData]);

  return (
    <div className="min-h-screen bg-background p-6 md:p-8">
      {/* Header */}
      <motion.header
        className="mb-6"
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.3 }}
      >
        <BentoCard elevation={3} background="glass" className="!p-4">
          <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
            <div>
              <h1 className="text-headline-md font-medium text-primary mb-1">
                FFNodes Dashboard
              </h1>
              <p className="text-body-md text-foreground/70">
                Real-time distributed encoding monitoring
              </p>
            </div>

            {/* Quick stats */}
            {systemStatus && (
              <div className="flex gap-4 text-center">
                <div>
                  <p className="text-title-lg font-medium text-primary">
                    {systemStatus.active_jobs}
                  </p>
                  <p className="text-body-sm text-foreground/60">Active</p>
                </div>
                <div>
                  <p className="text-title-lg font-medium text-foreground">
                    {systemStatus.pending_jobs}
                  </p>
                  <p className="text-body-sm text-foreground/60">Pending</p>
                </div>
                <div>
                  <p className="text-title-lg font-medium text-success">
                    {systemStatus.connected_clients}
                  </p>
                  <p className="text-body-sm text-foreground/60">Clients</p>
                </div>
              </div>
            )}
          </div>
        </BentoCard>
      </motion.header>

      {/* Main Dashboard Grid */}
      <BentoGrid columns={6} gap="md">
        {/* Overall Statistics - Full Width */}
        <OverallStatsBento />

        {/* Current Jobs - Left Side */}
        <CurrentJobsBento />

        {/* Connected Clients - Right Side */}
        <ConnectedClientsBento />

        {/* Leaderboard - Full Width */}
        <LeaderboardBento />

        {/* System Info Card */}
        <BentoCard colSpan={3} elevation={2} background="gradient">
          <BentoCardHeader
            title="System Information"
            icon={<iconify-icon icon="mdi:information" class="text-2xl" />}
          />
          <BentoCardContent>
            <div className="space-y-2 mt-2">
              <div className="flex justify-between items-center">
                <span className="text-body-md text-foreground/70">Server Status</span>
                <span className="flex items-center gap-2 text-body-md text-success">
                  <div className="w-2 h-2 rounded-full bg-success animate-pulse" />
                  Online
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-body-md text-foreground/70">Dashboard Version</span>
                <span className="text-body-md text-foreground">v0.1.21-beta</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-body-md text-foreground/70">Last Updated</span>
                <span className="text-body-md text-foreground">
                  {new Date().toLocaleTimeString()}
                </span>
              </div>
            </div>
          </BentoCardContent>
        </BentoCard>

        {/* Quick Actions Card */}
        <BentoCard colSpan={3} elevation={2} background="solid">
          <BentoCardHeader
            title="About FFNodes"
            icon={<iconify-icon icon="mdi:information-outline" class="text-2xl" />}
          />
          <BentoCardContent>
            <p className="text-body-md text-foreground/80 mt-2">
              FFNodes is a distributed video encoding system that leverages FFmpeg
              across multiple devices for efficient media processing.
            </p>
            <div className="mt-4 flex gap-2">
              <a
                href="https://github.com/Drew-Chase/FFNodes"
                target="_blank"
                rel="noopener noreferrer"
                className="text-primary hover:underline text-body-sm"
              >
                GitHub Repository →
              </a>
            </div>
          </BentoCardContent>
        </BentoCard>
      </BentoGrid>
    </div>
  );
}
