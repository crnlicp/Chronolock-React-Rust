import { Fragment, useState } from 'react';

interface IMobileNavigationProps {
  navigation: boolean;
  onNavigationToggle: (value: boolean) => void;
}

export const MobileNavigation = ({
  navigation,
  onNavigationToggle,
}: IMobileNavigationProps) => {
  const [showMobileMenu, setShowMobileMenu] = useState(false);
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
            <a
              href="#"
              onClick={(e) => {
                e.preventDefault();
              }}
              className="metaportal_fn_button wallet_opener"
            >
              <img src="assets/svg/ii.svg" width={150} height={50} />
            </a>
          </div>
        </div>
        <div className="mob_mid">
          <div className="logo">
            <a href="/">
              <img src="/img/logo.png" alt="" />
            </a>
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
              <a className="creative_link" href="#home">
                Home
              </a>
            </li>
            <li>
              <a className="creative_link" href="#about">
                About
              </a>
            </li>
            <li>
              <a className="creative_link" href="#collection">
                Collection
              </a>
            </li>
            <li>
              <a className="creative_link" href="#news">
                Blog
              </a>
            </li>
            <li>
              <a className="creative_link" href="#contact">
                Contact
              </a>
            </li>
          </ul>
        </div>
      </div>
    </Fragment>
  );
};
