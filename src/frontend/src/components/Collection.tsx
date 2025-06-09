import { useEffect } from 'react';
import Clock from './Clock';
import { NavLink } from 'react-router';

export const Collection = () => {
  useEffect(() => {
    const collection = document.querySelector('.fn_cs_collection');
    var items = collection?.querySelectorAll('.item') ?? [];
    var itemsLength = items?.length ?? 0;
    setInterval(function () {
      var numberOne = Math.floor(Math.random() * itemsLength);
      var numberTwo = Math.floor(Math.random() * itemsLength);

      while (numberTwo === numberOne) {
        numberTwo = Math.floor(Math.random() * itemsLength);
      }
      var firstDiv = items[numberOne];
      var secondDiv = items[numberTwo];
      firstDiv.classList.add('ready');
      secondDiv.classList.add('ready');
      setTimeout(function () {
        firstDiv.classList.remove('ready');
        secondDiv.classList.remove('ready');
      }, 500);
    }, 2000);
  }, []);

  const targetDate = new Date();
  targetDate.setHours(targetDate.getHours() + 2325);

  return (
    <section id="collection">
      <div className="container">
        <h3
          className="fn__maintitle big"
          data-text="Collection"
          data-align="center"
        >
          Collection
        </h3>
        <div className="fn_cs_collection">
          <div className="collection_top">
            <div className="item">
              <div className="item_in">
                <div className="img">
                  <div data-bg-img="assets/img/lock.png" className="abs_img" />
                  <Clock targetDate={targetDate} className="abs_img" />
                  <img src="assets/img/1x1.jpg" alt="" />
                </div>
              </div>
              <input type="hidden" defaultValue="assets/img/collection/1.jpg" />
            </div>

            <div className="item">
              <div className="item_in">
                <div className="img">
                  <div data-bg-img="assets/img/lock.png" className="abs_img" />
                  <Clock targetDate={targetDate} className="abs_img" />
                  <img src="assets/img/1x1.jpg" alt="" />
                </div>
              </div>
              <input type="hidden" defaultValue="assets/img/collection/1.jpg" />
            </div>

            <div className="item">
              <div className="item_in">
                <div className="img">
                  <div data-bg-img="assets/img/lock.png" className="abs_img" />
                  <Clock targetDate={targetDate} className="abs_img" />
                  <img src="assets/img/1x1.jpg" alt="" />
                </div>
              </div>
              <input type="hidden" defaultValue="assets/img/collection/1.jpg" />
            </div>

            <div className="item">
              <div className="item_in">
                <div className="img">
                  <div data-bg-img="assets/img/lock.png" className="abs_img" />
                  <Clock targetDate={targetDate} className="abs_img" />
                  <img src="assets/img/1x1.jpg" alt="" />
                </div>
              </div>
              <input type="hidden" defaultValue="assets/img/collection/1.jpg" />
            </div>
          </div>
          <div className="collection_bottom">
            <div className="item">
              <div className="item_in">
                <div className="img">
                  <div data-bg-img="assets/img/lock.png" className="abs_img" />
                  <Clock targetDate={targetDate} className="abs_img" />
                  <img src="assets/img/1x1.jpg" alt="" />
                </div>
              </div>
              <input type="hidden" defaultValue="assets/img/collection/1.jpg" />
            </div>

            <div className="item">
              <div className="item_in">
                <div className="img">
                  <div data-bg-img="assets/img/lock.png" className="abs_img" />
                  <Clock targetDate={targetDate} className="abs_img" />
                  <img src="assets/img/1x1.jpg" alt="" />
                </div>
              </div>
              <input type="hidden" defaultValue="assets/img/collection/1.jpg" />
            </div>

            <div className="item">
              <div className="item_in">
                <div className="img">
                  <div data-bg-img="assets/img/lock.png" className="abs_img" />
                  <Clock targetDate={targetDate} className="abs_img" />
                  <img src="assets/img/1x1.jpg" alt="" />
                </div>
              </div>
              <input type="hidden" defaultValue="assets/img/collection/1.jpg" />
            </div>

            <div className="item">
              <div className="item_in">
                <div className="img">
                  <div data-bg-img="assets/img/lock.png" className="abs_img" />
                  <Clock targetDate={targetDate} className="abs_img" />
                  <img src="assets/img/1x1.jpg" alt="" />
                </div>
              </div>
              <input type="hidden" defaultValue="assets/img/collection/1.jpg" />
            </div>
          </div>
        </div>
        <div className="fn_cs_desc">
          <p>
            Meta Legends represent a collection of 17,000 legends categorized by
            level of rarity and generated with hundreds of elements. The Legends
            are stored as ERC-721 tokens on the Ethereum blockchain and hosted
            on IPFS.
          </p>
          <NavLink to="/collection" className="metaportal_fn_button">
            <span>See Collection</span>
          </NavLink>
        </div>
      </div>
    </section>
  );
};
