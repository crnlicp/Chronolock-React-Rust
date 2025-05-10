import { useEffect } from 'react';
import { stickyNav } from '../../utils/utility';
import { useMenuClick } from '../../hooks/useMenuClick';
import { NavLink } from 'react-router';

interface IHeaderProps {
  onNavigationToggle: (value: boolean) => void;
}

export const MainNavigation = ({ onNavigationToggle }: IHeaderProps) => {
  useEffect(() => {
    stickyNav();
  }, []);

  useMenuClick();

  return (
    <header id="header">
      <div className="header">
        <div className="header_in">
          <div className="trigger_logo">
            <div className="trigger" onClick={() => onNavigationToggle(true)}>
              <span />
            </div>
            <div className="logo">
              <NavLink to="/">
                <img src="assets/img/logo.png" alt="" />
              </NavLink>
            </div>
          </div>
          <div className="nav" style={{ opacity: 1 }}>
            <ul>
              <li>
                <NavLink
                  to="/#home"
                  className="creative_link"
                  //
                >
                  Home
                </NavLink>
              </li>
              <li>
                <NavLink to="/#about" className="creative_link">
                  About
                </NavLink>
              </li>
              <li>
                <NavLink to="/#collection" className="creative_link">
                  Collection
                </NavLink>
              </li>
              <li>
                <NavLink to="/#roadmap" className="creative_link">
                  Road Map
                </NavLink>
              </li>
              <li>
                <NavLink to="/#faq" className="creative_link">
                  FAQ
                </NavLink>
              </li>
            </ul>
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
      </div>
    </header>
  );
};
