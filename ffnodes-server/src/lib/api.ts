import type {
  OverallSystemStats,
  SystemStatus,
  ClientStatus,
  EncodingJob,
  LeaderboardResponse,
  LeaderboardCategory,
} from '../types/api';

// Base URL - empty for production (same origin), proxy handles it in dev
const BASE_URL = '';

/**
 * Fetch overall system statistics
 * GET /api/public/stats/overall
 */
export async function fetchOverallStats(): Promise<OverallSystemStats> {
  const res = await fetch(`${BASE_URL}/api/public/stats/overall`);
  if (!res.ok) {
    throw new Error(`Failed to fetch overall stats: ${res.statusText}`);
  }
  return res.json();
}

/**
 * Fetch system status (pending/active jobs, connected clients)
 * GET /api/public/monitoring/status
 */
export async function fetchSystemStatus(): Promise<SystemStatus> {
  const res = await fetch(`${BASE_URL}/api/public/monitoring/status`);
  if (!res.ok) {
    throw new Error(`Failed to fetch system status: ${res.statusText}`);
  }
  return res.json();
}

/**
 * Fetch all connected clients with their statuses
 * GET /api/public/monitoring/clients
 */
export async function fetchConnectedClients(): Promise<ClientStatus[]> {
  const res = await fetch(`${BASE_URL}/api/public/monitoring/clients`);
  if (!res.ok) {
    throw new Error(`Failed to fetch connected clients: ${res.statusText}`);
  }
  return res.json();
}

/**
 * Fetch active encoding jobs
 * GET /api/public/jobs/active
 */
export async function fetchActiveJobs(): Promise<EncodingJob[]> {
  const res = await fetch(`${BASE_URL}/api/public/jobs/active`);
  if (!res.ok) {
    throw new Error(`Failed to fetch active jobs: ${res.statusText}`);
  }
  return res.json();
}

/**
 * Fetch leaderboard data by category
 * GET /api/public/stats/leaderboard?category={category}
 */
export async function fetchLeaderboard(
  category: LeaderboardCategory
): Promise<LeaderboardResponse> {
  const res = await fetch(`${BASE_URL}/api/public/stats/leaderboard?category=${category}`);
  if (!res.ok) {
    throw new Error(`Failed to fetch leaderboard: ${res.statusText}`);
  }
  return res.json();
}
