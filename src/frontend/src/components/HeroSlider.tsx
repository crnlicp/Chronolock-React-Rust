import { useEffect } from 'react';
import WaterWave from 'react-water-wave';
import Clock from './Clock';
import { NavLink } from 'react-router';

export const HeroSlider = ({ targetDate }: { targetDate: Date }) => {
  useEffect(() => {
    const fn_cs_slider = document.querySelectorAll('.fn_cs_slider');
    fn_cs_slider.forEach((element) => {
      let sliderTop = element.getElementsByClassName('slider_top')[0],
        sliderBottom = element.getElementsByClassName('slider_content'),
        activeIndex = 2,
        speed = 6000;

      let myInterval = setInterval(function () {
        activeIndex++;
        activeIndex = sliderDo(sliderTop, sliderBottom, activeIndex);
      }, speed);
      const prev = document.querySelector('.slider_nav .prev'),
        next = document.querySelector('.slider_nav .next'),
        li = element.getElementsByTagName('li');
      prev?.addEventListener('click', function (e) {
        e.preventDefault();
        clearInterval(myInterval);
        activeIndex--;
        activeIndex = sliderDo(sliderTop, sliderBottom, activeIndex);
        myInterval = setInterval(function () {
          activeIndex++;
          activeIndex = sliderDo(sliderTop, sliderBottom, activeIndex);
        }, speed);
        return false;
      });
      next?.addEventListener('click', (e) => {
        e.preventDefault();
        clearInterval(myInterval);
        activeIndex++;
        activeIndex = sliderDo(sliderTop, sliderBottom, activeIndex);
        myInterval = setInterval(function () {
          activeIndex--;
          activeIndex = sliderDo(sliderTop, sliderBottom, activeIndex);
        }, speed);
        return false;
      });
      for (let i = 0; i < li.length; i++) {
        const liElement = li[i];
        const getClass = liElement.getAttribute('class');
        if (getClass === 'next') {
          activeIndex++;
        } else if (getClass === 'prev') {
          activeIndex--;
        } else {
          return false;
        }
        clearInterval(myInterval);
        activeIndex = sliderDo(sliderTop, sliderBottom, activeIndex);
        myInterval = setInterval(function () {
          activeIndex++;
          activeIndex = sliderDo(sliderTop, sliderBottom, activeIndex);
        }, speed);
        return false;
      }
    });
  }, []);

  const sliderDo = (
    sliderTop: Element,
    _sliderBottom: HTMLCollectionOf<Element>,
    activeIndex: number,
  ) => {
    var topLength = sliderTop.getElementsByTagName('li').length;
    if (activeIndex > topLength) {
      activeIndex -= topLength;
    }
    var indexPrev = activeIndex - 1;
    var indexPrev2 = activeIndex - 2;
    var indexNext = activeIndex + 1;
    var indexNext2 = activeIndex + 2;
    if (indexPrev > topLength) {
      indexPrev -= topLength;
    }
    if (indexPrev2 > topLength) {
      indexPrev2 -= topLength;
    }
    if (indexNext > topLength) {
      indexNext -= topLength;
    }
    if (indexNext2 > topLength) {
      indexNext2 -= topLength;
    }
    if (indexPrev < 1) {
      indexPrev += topLength;
    }
    if (indexPrev2 < 1) {
      indexPrev2 += topLength;
    }
    if (activeIndex < 1) {
      activeIndex += topLength;
    }
    if (indexNext < 1) {
      indexNext += topLength;
    }
    if (indexNext2 < 1) {
      indexNext2 += topLength;
    }
    let li = sliderTop.getElementsByTagName('li');
    for (let i = 0; i < li.length; i++) {
      const element = li[i];
      element.classList.remove('prev', 'prev2', 'active', 'next', 'next2');
      // element.setAttribute(`data-index${indexNext}`);
    }
    sliderTop
      ?.querySelector('li[data-index="' + indexPrev2 + '"]')
      ?.classList.add('prev2');
    sliderTop
      ?.querySelector('li[data-index="' + indexPrev + '"]')
      ?.classList.add('prev');
    sliderTop
      ?.querySelector('li[data-index="' + activeIndex + '"]')
      ?.classList.add('active');
    sliderTop
      ?.querySelector('li[data-index="' + indexNext + '"]')
      ?.classList.add('next');
    sliderTop
      ?.querySelector('li[data-index="' + indexNext2 + '"]')
      ?.classList.add('next2');
    return activeIndex;
  };

  return (
    <WaterWave
      imageUrl="assets/img/wwpeakpx.jpg"
      style={{
        width: '100%',
        height: '100%',
        backgroundSize: 'cover',
        backgroundPosition: 'center',
      }}
      perturbance={0.005}
    >
      {({ updateSize }: { updateSize: () => void }) => {
        useEffect(() => {
          window.addEventListener('resize', updateSize);
          const timeout = setTimeout(() => {
            updateSize();
          }, 0);
          return () => {
            window.removeEventListener('resize', updateSize);
            clearTimeout(timeout);
          };
        }, [updateSize]);

        return (
          <section id="home">
            <div className="container">
              <h3
                className="fn__maintitle big"
                data-text="Chronolock"
                data-align="center"
              >
                Chronolock
              </h3>
              {/* Slider */}
              <div className="fn_cs_slider" data-responsive="on">
                <div className="slider_top">
                  <img src="/assets/img/1x1.jpg" alt="" />
                  <ul>
                    <li className="prev" data-index={1}>
                      <div className="item">
                        <img src="/assets/img/1x1.jpg" alt="" />
                        <div className="item_in">
                          <div
                            className="img"
                            data-bg-img="assets/img/lock.png"
                          >
                            <Clock targetDate={targetDate} />
                          </div>
                        </div>
                      </div>
                    </li>
                    <li className="active" data-index={2}>
                      <div className="item">
                        <img src="/assets/img/1x1.jpg" alt="" />
                        <div className="item_in">
                          <div
                            className="img"
                            data-bg-img="assets/img/lock.png"
                          >
                            <Clock targetDate={targetDate} />
                          </div>
                        </div>
                      </div>
                    </li>
                    <li className="next" data-index={3}>
                      <div className="item">
                        <img src="/assets/img/1x1.jpg" alt="" />
                        <div className="item_in">
                          <div
                            className="img"
                            data-bg-img="assets/img/lock.png"
                          >
                            <Clock targetDate={targetDate} />
                          </div>
                        </div>
                      </div>
                    </li>
                    <li className="next2" data-index={4}>
                      <div className="item">
                        <img src="assets/img/1x1.jpg" alt="" />
                        <div className="item_in">
                          <div
                            className="img"
                            data-bg-img="assets/img/lock.png"
                          >
                            <Clock targetDate={targetDate} />
                          </div>
                        </div>
                      </div>
                    </li>
                    <li data-index={5}>
                      <div className="item">
                        <img src="assets/img/1x1.jpg" alt="" />
                        <div className="item_in">
                          <div
                            className="img"
                            data-bg-img="assets/img/lock.png"
                          >
                            <Clock targetDate={targetDate} />
                          </div>
                        </div>
                      </div>
                    </li>
                    <li data-index={6}>
                      <div className="item">
                        <img src="assets/img/1x1.jpg" alt="" />
                        <div className="item_in">
                          <div
                            className="img"
                            data-bg-img="assets/img/lock.png"
                          >
                            <Clock targetDate={targetDate} />
                          </div>
                        </div>
                      </div>
                    </li>
                    <li className="prev2" data-index={7}>
                      <div className="item">
                        <img src="assets/img/1x1.jpg" alt="" />
                        <div className="item_in">
                          <div
                            className="img"
                            data-bg-img="assets/img/lock.png"
                          >
                            <Clock targetDate={targetDate} />
                          </div>
                        </div>
                      </div>
                    </li>
                  </ul>
                </div>
                <div className="slider_nav">
                  <NavLink to="#" className="prev">
                    <span className="circle" />
                    <span className="icon">
                      <img
                        src="assets/svg/down.svg"
                        alt=""
                        className="fn__svg"
                      />
                    </span>
                    <span className="circle" />
                  </NavLink>
                  <NavLink to="#" className="next">
                    <span className="circle" />
                    <span className="icon">
                      <img
                        src="assets/svg/down.svg"
                        alt=""
                        className="fn__svg"
                      />
                    </span>
                    <span className="circle" />
                  </NavLink>
                </div>
              </div>
              {/* !Slider */}
              {/* Description */}
              <div className="fn_cs_desc">
                <p>
                  Chronolock makes it easy to lock and manage your digital
                  assets using the Internet Computer blockchain. With our
                  platform, you can set up secure time-lock assets and keep them
                  safe—all with a simple and user-friendly interface. Whether
                  you are new to blockchain or an experienced user, Chronolock
                  helps you protect and control your assets with advanced
                  technology and reliable security.
                </p>
              </div>
              {/* !Description */}
            </div>
          </section>
        );
      }}
    </WaterWave>
  );
};
