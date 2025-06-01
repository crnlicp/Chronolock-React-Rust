import { Fragment, useState } from 'react';
import { NavLink } from 'react-router';

interface IMobileNavigationProps {
  navigation: boolean;
  onNavigationToggle: (value: boolean) => void;
}

export const MobileNavigation = ({
  navigation,
  onNavigationToggle,
}: IMobileNavigationProps) => {
  const [showMobileMenu, setShowMobileMenu] = useState(false);

  const handleMobileMenuToggle = () => {
    setShowMobileMenu(!showMobileMenu);
  };
  return (
    <Fragment>
      <div className="metaportal_fn_mobnav">
        <div className="mob_top">
          <div className="social_trigger">
            <div
              className={`trigger ${navigation ? 'active' : ''}`}
              onClick={() => onNavigationToggle(true)}
            >
              <span />
            </div>
          </div>
          <div className="wallet">
            <NavLink
              to="#"
              onClick={(e) => {
                e.preventDefault();
              }}
              className="metaportal_fn_button wallet_opener"
            >
              <img src="assets/svg/ii.svg" width={150} height={50} />
            </NavLink>
          </div>
        </div>
        <div className="mob_mid">
          <div className="logo">
            <NavLink to="/">
              <img src="/img/logo.png" alt="" />
            </NavLink>
          </div>
          <div
            className={`trigger ${showMobileMenu ? 'active' : ''}`}
            onClick={() => setShowMobileMenu(!showMobileMenu)}
          >
            <span />
          </div>
        </div>
        <div
          className="mob_bot"
          style={{ display: showMobileMenu ? 'block' : 'none' }}
        >
          <ul>
            <li>
              <NavLink
                className="creative_link"
                to="/#home"
                onClick={handleMobileMenuToggle}
              >
                Home
              </NavLink>
            </li>
            <li>
              <NavLink
                className="creative_link"
                to="/#about"
                onClick={handleMobileMenuToggle}
              >
                About
              </NavLink>
            </li>
            <li>
              <NavLink
                className="creative_link"
                to="/#collection"
                onClick={handleMobileMenuToggle}
              >
                Collection
              </NavLink>
            </li>
            <li>
              <NavLink
                to="/#roadmap"
                className="creative_link"
                onClick={handleMobileMenuToggle}
              >
                Road Map
              </NavLink>
            </li>
            <li>
              <NavLink
                to="/#faq"
                className="creative_link"
                onClick={handleMobileMenuToggle}
              >
                FAQ
              </NavLink>
            </li>
          </ul>
        </div>
      </div>
    </Fragment>
  );
};
