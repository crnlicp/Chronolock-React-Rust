import { Fragment } from 'react';
import { useMenuClick } from '../../hooks/useMenuClick';
import { NavLink } from 'react-router';

interface INavigationProps {
  navigation: boolean;
  onNavigationToggle: (value: boolean) => void;
}

export const DrawerNavigation = ({
  navigation,
  onNavigationToggle,
}: INavigationProps) => {
  useMenuClick();

  const handleClick = () => {
    onNavigationToggle(false);
  };

  return (
    <Fragment>
      <div
        onClick={() => onNavigationToggle(false)}
        className={`metaportal_fn_leftnav_closer ${navigation ? 'active' : ''}`}
      />
      <div className={`metaportal_fn_leftnav ${navigation ? 'active' : ''}`}>
        <NavLink
          to="#"
          className="fn__closer"
          id="metaportal_fn_leftnav_closer"
          onClick={() => onNavigationToggle(false)}
        >
          <span />
        </NavLink>
        <div className="navbox">
          <div className="list_holder">
            <ul className="metaportal_fn_items">
              <li>
                <div className="item">
                  <img
                    src="/assets/img/logo.png"
                    alt=""
                    height={150}
                    style={{ margin: 20 }}
                  />
                  <span className="text">Chronolock</span>
                </div>
              </li>
              {/* <li>
                <div className="item">
                  <a
                    to="https://opensea.io/"
                    target="_blank"
                    rel="noreferrer"
                  />
                  <span className="icon">
                    <img src="/assets/img/market/opensea.png" alt="" />
                  </span>
                  <span className="text">Opensea</span>
                </div>
              </li> */}
              {/* <li>
                <div className="item">
                  <a
                    to="https://discord.com/"
                    target="_blank"
                    rel="noreferrer"
                  />
                  <span className="icon">
                    <img src="/assets/img/market/discord.png" alt="" />
                  </span>
                  <span className="text">Discord</span>
                </div>
              </li> */}
            </ul>
          </div>
          <div className="nav_holder">
            {/* For JS */}
            <span className="icon">
              <img src="/svg/down.svg" alt="" className="fn__svg" />
            </span>
            {/* For JS */}
            <ul>
              <li>
                <NavLink to="/#home" onClick={handleClick}>
                  <span className="creative_link">
                    Home
                    <img src="/svg/down.svg" alt="" className="fn__svg" />
                  </span>
                </NavLink>
              </li>
              <li>
                <NavLink to="/#about" onClick={handleClick}>
                  <span className="creative_link">
                    About
                    <img src="/svg/down.svg" alt="" className="fn__svg" />
                  </span>
                </NavLink>
              </li>
              <li>
                <NavLink to="/collection" onClick={handleClick}>
                  <span className="creative_link">
                    Collection
                    <img src="/svg/down.svg" alt="" className="fn__svg" />
                  </span>
                </NavLink>
              </li>
              <li>
                <NavLink to="/#roadmap" onClick={handleClick}>
                  <span className="creative_link">
                    Road Map
                    <img src="/svg/down.svg" alt="" className="fn__svg" />
                  </span>
                </NavLink>
              </li>
              <li>
                <NavLink to="/#faq" onClick={handleClick}>
                  <span className="creative_link">
                    FAQ
                    <img src="/svg/down.svg" alt="" className="fn__svg" />
                  </span>
                </NavLink>
              </li>
            </ul>
          </div>
          <div className="info_holder">
            <div className="copyright">
              <p>Copyright 2025 - Chronolock</p>
            </div>
          </div>
        </div>
      </div>
    </Fragment>
  );
};
