import { Counter } from './Counter';

export const FunFacts = () => {
  return (
    <section id="fun_facts">
      <div className="container">
        <div className="fn_cs_counter_list">
          <ul>
            <li>
              <div className="item">
                <h3 className="fn__gradient_title">
                  <span className="prefix" />
                  <Counter end={200} />
                  <span className="suffix" />
                </h3>
                <p>Total Items</p>
                <div className="divider" />
              </div>
            </li>
            <li>
              <div className="item">
                <h3 className="fn__gradient_title">
                  <span className="prefix" />
                  <Counter end={1} decimals={2} />
                  <span className="suffix">k+</span>
                </h3>
                <p>Total Owners</p>
                <div className="divider" />
              </div>
            </li>
            <li>
              <div className="item">
                <h3 className="fn__gradient_title">
                  <span className="prefix" />
                  <Counter end={2.4} decimals={2} />
                  <span className="suffix" />
                </h3>
                <p>Floor Price (ETH)</p>
                <div className="divider" />
              </div>
            </li>
            <li>
              <div className="item">
                <h3 className="fn__gradient_title">
                  <span className="prefix" />
                  <Counter end={144} />
                  <span className="suffix">k+</span>
                </h3>
                <p>Volume Traded (ETH)</p>
                <div className="divider" />
              </div>
            </li>
          </ul>
        </div>
      </div>
    </section>
  );
};
