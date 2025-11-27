import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { motion } from 'framer-motion';
import { BentoCard, BentoCardHeader, BentoCardContent } from '../layout/BentoGrid';
import { Button } from '../ui';
import { useConfigStore } from '../../stores/useConfigStore';

interface LeaderboardEntry {
  rank: number;
  client_name: string;
  value: number;
  formatted_value: string;
}

interface LeaderboardResponse {
  category: string;
  entries: LeaderboardEntry[];
}

type LeaderboardCategory = 'most_jobs' | 'most_saved' | 'highest_speed';

interface CategoryInfo {
  id: LeaderboardCategory;
  label: string;
  icon: string;
  color: string;
}

const categories: CategoryInfo[] = [
  { id: 'most_jobs', label: 'Most Jobs', icon: 'mdi:briefcase-check', color: 'primary' },
  { id: 'most_saved', label: 'Most Saved', icon: 'mdi:content-save', color: 'success' },
  { id: 'highest_speed', label: 'Fastest', icon: 'mdi:speedometer', color: 'warning' },
];

export function LeaderboardBento() {
  const { config } = useConfigStore();
  const [selectedCategory, setSelectedCategory] = useState<LeaderboardCategory>('most_jobs');
  const [leaderboard, setLeaderboard] = useState<LeaderboardResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchLeaderboard = async () => {
      if (!config) return;

      try {
        setLoading(true);
        const data: LeaderboardResponse = await invoke('get_leaderboard', {
          config,
          category: selectedCategory,
        });
        setLeaderboard(data);
        setError(null);
      } catch (err) {
        setError(String(err));
        console.error('Failed to fetch leaderboard:', err);
      } finally {
        setLoading(false);
      }
    };

    fetchLeaderboard();
    // Refresh every 60 seconds
    const interval = setInterval(fetchLeaderboard, 60000);
    return () => clearInterval(interval);
  }, [config, selectedCategory]);

  const currentCategory = categories.find((c) => c.id === selectedCategory);
  const getRankIcon = (rank: number) => {
    switch (rank) {
      case 1:
        return 'mdi:trophy';
      case 2:
        return 'mdi:medal';
      case 3:
        return 'mdi:podium-bronze';
      default:
        return 'mdi:numeric-' + rank + '-circle';
    }
  };

  const getRankColor = (rank: number) => {
    switch (rank) {
      case 1:
        return 'text-warning';
      case 2:
        return 'text-default-400';
      case 3:
        return 'text-orange-600';
      default:
        return 'text-foreground/60';
    }
  };

  if (loading) {
    return (
      <BentoCard colSpan={2} rowSpan={2} elevation={3} background="glass">
        <BentoCardHeader
          title="Leaderboard"
          icon={<iconify-icon icon="mdi:trophy-variant" class="text-2xl" />}
        />
        <BentoCardContent>
          <div className="flex items-center justify-center py-8">
            <div className="neon-spinner" />
          </div>
        </BentoCardContent>
      </BentoCard>
    );
  }

  return (
    <BentoCard colSpan={2} rowSpan={2} elevation={3} background="glass" className="overflow-hidden">
      <BentoCardHeader
        title="Leaderboard"
        subtitle={currentCategory?.label}
        icon={<iconify-icon icon="mdi:trophy-variant" class="text-2xl" />}
      />

      {/* Category Selector */}
      <div className="flex gap-2 mb-4">
        {categories.map((category) => (
          <Button
            key={category.id}
            size="sm"
            variant={selectedCategory === category.id ? 'primary' : 'accent'}
            onPress={() => setSelectedCategory(category.id)}
            className="flex-1"
          >
            <iconify-icon icon={category.icon} class="mr-1" />
            <span className="hidden md:inline">{category.label}</span>
          </Button>
        ))}
      </div>

      {error ? (
        <BentoCardContent>
          <div className="text-center text-danger py-4">
            <p>Failed to load leaderboard</p>
            <p className="text-sm mt-2">{error}</p>
          </div>
        </BentoCardContent>
      ) : !leaderboard || leaderboard.entries.length === 0 ? (
        <BentoCardContent>
          <div className="text-center text-foreground/60 py-8">
            <iconify-icon icon="mdi:trophy-broken" class="text-6xl mb-4 opacity-50" />
            <p>No data available</p>
          </div>
        </BentoCardContent>
      ) : (
        <BentoCardContent>
          <div className="space-y-2 max-h-[450px] overflow-y-auto pr-2">
            {leaderboard.entries.map((entry, idx) => (
              <motion.div
                key={idx}
                className="bg-content2 rounded-md p-3 hover:bg-content2/80 transition-colors"
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ duration: 0.3, delay: idx * 0.05 }}
              >
                <div className="flex items-center gap-3">
                  {/* Rank Icon */}
                  <div className={`flex-shrink-0 ${getRankColor(entry.rank)}`}>
                    <iconify-icon icon={getRankIcon(entry.rank)} class="text-3xl" />
                  </div>

                  {/* Client Name */}
                  <div className="flex-1 min-w-0">
                    <p className="font-medium text-foreground truncate">{entry.client_name}</p>
                  </div>

                  {/* Value */}
                  <div className={`flex-shrink-0 text-right text-${currentCategory?.color}`}>
                    <p className="text-lg font-bold">{entry.formatted_value}</p>
                  </div>
                </div>
              </motion.div>
            ))}
          </div>

          {/* Encourage participation */}
          {leaderboard.entries.length > 0 && (
            <div className="mt-4 pt-4 border-t border-divider text-center">
              <p className="text-xs text-foreground/60">
                Keep encoding to climb the leaderboard!
              </p>
            </div>
          )}
        </BentoCardContent>
      )}
    </BentoCard>
  );
}
