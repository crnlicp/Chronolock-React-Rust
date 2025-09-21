import { useEffect, useState } from 'react';
import { useChronolock } from './useChronolock';
import { useCrnlToken } from './useCrnlToken';

export interface FunFactsStats {
  totalChronolocks: number;
  uniqueCreators: number;
  totalSupply: number;
  totalBurned: number;
}

interface UseFunFactsStatsReturn {
  stats: FunFactsStats;
  isLoading: boolean;
  error: Error | null;
  refetch: () => void;
}

export const useFunFactsStats = (): UseFunFactsStatsReturn => {
  const [stats, setStats] = useState<FunFactsStats>({
    totalChronolocks: 0,
    uniqueCreators: 0,
    totalSupply: 0,
    totalBurned: 0,
  });
  const [error, setError] = useState<Error | null>(null);

  const {
    getAllChronolocksCount,
    getUniqueCreatorsCount,
    isGetAllChronolocksCountLoading,
    isGetUniqueCreatorsCountLoading,
  } = useChronolock();

  const {
    getTotalSupply,
    getTotalBurned,
    isTotalSupplyLoading,
    isTotalBurnedLoading,
    totalSupplyData,
    totalBurnedData,
  } = useCrnlToken();

  const isLoading =
    isGetAllChronolocksCountLoading ||
    isGetUniqueCreatorsCountLoading ||
    isTotalSupplyLoading ||
    isTotalBurnedLoading;

  const formatTokenAmount = (amount: unknown): number => {
    if (typeof amount === 'bigint') {
      return Number(amount) / 1e8;
    }
    if (typeof amount === 'number') {
      return amount / 1e8;
    }
    if (typeof amount === 'string') {
      return parseFloat(amount) / 1e8;
    }
    return 0;
  };

  const fetchStats = async () => {
    try {
      setError(null);

      // Fetch chronolock statistics
      const [chronolocksCount, creatorsCount] = await Promise.all([
        getAllChronolocksCount(),
        getUniqueCreatorsCount(),
      ]);

      // Token statistics are already being fetched automatically by the hooks

      setStats((prevStats) => ({
        ...prevStats,
        totalChronolocks:
          typeof chronolocksCount === 'bigint'
            ? Number(chronolocksCount)
            : (chronolocksCount as number) || 0,
        uniqueCreators:
          typeof creatorsCount === 'bigint'
            ? Number(creatorsCount)
            : (creatorsCount as number) || 0,
      }));
    } catch (err) {
      setError(err as Error);
      console.error('Error fetching fun facts stats:', err);
    }
  };

  // Update token stats when data changes
  useEffect(() => {
    if (totalSupplyData || totalBurnedData) {
      setStats((prevStats) => ({
        ...prevStats,
        totalSupply: formatTokenAmount(totalSupplyData),
        totalBurned: formatTokenAmount(totalBurnedData),
      }));
    }
  }, [totalSupplyData, totalBurnedData]);

  // Initial fetch
  useEffect(() => {
    fetchStats();
  }, []); // Only run once on mount

  const refetch = () => {
    fetchStats();
    // Also trigger refetch of token data
    getTotalSupply();
    getTotalBurned();
  };

  return {
    stats,
    isLoading,
    error,
    refetch,
  };
};
