import { Counter } from './Counter';
import { useFunFactsStats } from '../hooks/useFunFactsStats';

export const FunFacts = () => {
  const { stats, isLoading, error } = useFunFactsStats();

  // Format large numbers for display
  const formatNumber = (
    num: number,
  ): { value: number; suffix: string; decimals: number } => {
    if (num >= 1000000000000) {
      return { value: num / 1000000000000, suffix: ' T', decimals: 1 };
    }
    if (num >= 1000000000) {
      return { value: num / 1000000000, suffix: ' B', decimals: 1 };
    }
    if (num >= 1000000) {
      return { value: num / 1000000, suffix: ' M', decimals: 1 };
    }
    if (num >= 1000) {
      return { value: num / 1000, suffix: ' K', decimals: 1 };
    }
    return { value: num, suffix: '', decimals: 0 };
  };

  const totalChronolocks = formatNumber(stats.totalChronolocks);
  const uniqueCreators = formatNumber(stats.uniqueCreators);
  const totalSupply = formatNumber(stats.totalSupply);
  const totalBurned = formatNumber(stats.totalBurned);

  if (error) {
    console.error('Error loading fun facts:', error);
  }

  // Show placeholder values while loading
  const displayStats = isLoading
    ? {
        totalChronolocks: { value: 0, suffix: '', decimals: 0 },
        uniqueCreators: { value: 0, suffix: '', decimals: 0 },
        totalSupply: { value: 0, suffix: '', decimals: 0 },
        totalBurned: { value: 0, suffix: '', decimals: 0 },
      }
    : {
        totalChronolocks,
        uniqueCreators,
        totalSupply,
        totalBurned,
      };
  return (
    <section id="fun_facts">
      <div className="container">
        <div className="fn_cs_counter_list">
          <ul>
            <li>
              <div className="item">
                <h3 className="fn__gradient_title">
                  <span className="prefix" />
                  <Counter
                    end={displayStats.totalChronolocks.value}
                    decimals={displayStats.totalChronolocks.decimals}
                  />
                  <span className="suffix">
                    {displayStats.totalChronolocks.suffix}
                  </span>
                </h3>
                <p>Total Chronolocks</p>
                <div className="divider" />
              </div>
            </li>
            <li>
              <div className="item">
                <h3 className="fn__gradient_title">
                  <span className="prefix" />
                  <Counter
                    end={displayStats.uniqueCreators.value}
                    decimals={displayStats.uniqueCreators.decimals}
                  />
                  <span className="suffix">
                    {displayStats.uniqueCreators.suffix}
                  </span>
                </h3>
                <p>Unique Creators</p>
                <div className="divider" />
              </div>
            </li>
            <li>
              <div className="item">
                <h3 className="fn__gradient_title">
                  <span className="prefix" />
                  <Counter
                    end={displayStats.totalSupply.value}
                    decimals={displayStats.totalSupply.decimals}
                  />
                  <span className="suffix">
                    {displayStats.totalSupply.suffix}
                  </span>
                </h3>
                <p>Total $CRNL Supply</p>
                <div className="divider" />
              </div>
            </li>
            <li>
              <div className="item">
                <h3 className="fn__gradient_title">
                  <span className="prefix" />
                  <Counter
                    end={displayStats.totalBurned.value}
                    decimals={displayStats.totalBurned.decimals}
                  />
                  <span className="suffix">
                    {displayStats.totalBurned.suffix}
                  </span>
                </h3>
                <p>Total $CRNL Burned</p>
                <div className="divider" />
              </div>
            </li>
          </ul>
        </div>
      </div>
    </section>
  );
};
