import { useEffect, useState } from 'react';
import { stickyNav } from '../../utils/utility';
import { useMenuClick } from '../../hooks/useMenuClick';
import { NavLink } from 'react-router';

import LogoutIcon from '@mui/icons-material/Logout';
import ContentCopyIcon from '@mui/icons-material/ContentCopy';
import CheckBoxIcon from '@mui/icons-material/CheckBox';
import { Backdrop, CircularProgress } from '@mui/material';
import { useAuth } from '../../hooks/useAuth';

interface IHeaderProps {
  onNavigationToggle: (value: boolean) => void;
}

export const MainNavigation = ({ onNavigationToggle }: IHeaderProps) => {
  const {
    isAuthenticated,
    principal,
    isAuthLoading,
    handleLogin,
    handleLogout,
  } = useAuth();

  const [copied, setCopied] = useState(false);

  useEffect(() => {
    stickyNav();
  }, []);

  useMenuClick();

  function formatPrincipal(principal: string) {
    if (principal.length <= 8) return principal;
    return `${principal.slice(0, 5)}...${principal.slice(-3)}`;
  }

  function handleCopy() {
    if (principal) {
      navigator.clipboard.writeText(principal);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  }

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
            {isAuthenticated ? (
              <div style={{ display: 'flex', alignItems: 'center', gap: 25 }}>
                <span
                  title={principal || ''}
                  style={{ fontFamily: 'monospace' }}
                  onClick={handleCopy}
                >
                  {principal ? formatPrincipal(principal) : ''}
                </span>
                <button
                  onClick={handleCopy}
                  style={{
                    background: 'none',
                    border: 'none',
                    cursor: 'pointer',
                    padding: 0,
                  }}
                  title="Copy Principal"
                >
                  {!copied ? (
                    <ContentCopyIcon
                      sx={{
                        color: copied ? 'lightGreen' : 'gray',
                      }}
                    />
                  ) : (
                    <span
                      style={{
                        display: 'flex',
                        flexDirection: 'column',
                        alignItems: 'center',
                        gap: 5,
                      }}
                    >
                      <CheckBoxIcon
                        sx={{
                          color: copied ? 'lightGreen' : 'transparent',
                        }}
                      />
                      <span
                        style={{
                          fontSize: '0.8em',
                          color: 'lightGreen',
                          position: 'absolute',
                          top: '65%',
                        }}
                      >
                        Copied!
                      </span>
                    </span>
                  )}
                </button>
                <button
                  onClick={handleLogout}
                  className="metaportal_fn_button"
                  style={{
                    fontSize: '0.9em',
                    padding: '0.3em 0.8em',
                    border: 'none',
                  }}
                >
                  <LogoutIcon />
                </button>
              </div>
            ) : (
              <NavLink
                to="#"
                onClick={(e) => {
                  e.preventDefault();
                  handleLogin();
                }}
                className="metaportal_fn_button wallet_opener"
              >
                <img src="assets/svg/ii.svg" width={150} height={50} />
              </NavLink>
            )}
          </div>
        </div>
      </div>
      <Backdrop
        sx={(theme) => ({ color: '#fff', zIndex: theme.zIndex.drawer + 1 })}
        open={isAuthLoading}
      >
        <CircularProgress />
      </Backdrop>
    </header>
  );
};
