import { Fragment, useState } from 'react';
import {
  Box,
  ClickAwayListener,
  IconButton,
  styled,
  Tooltip,
  tooltipClasses,
  TooltipProps,
} from '@mui/material';
import KeyboardArrowDownRoundedIcon from '@mui/icons-material/KeyboardArrowDownRounded';
import PersonRoundedIcon from '@mui/icons-material/PersonRounded';
import { NavLink } from 'react-router';
import { useAuth } from '../../hooks/useAuth';
import { IUseCrnlToken } from '../../hooks/useCrnlToken';
import { UserMenu } from './UserMenu';

interface IMobileNavigationProps {
  navigation: boolean;
  crnlTokenHook: IUseCrnlToken;
  onNavigationToggle: (value: boolean) => void;
}

const CustomTooltip = styled(({ className, ...props }: TooltipProps) => (
  <Tooltip {...props} classes={{ popper: className }} />
))(() => ({
  [`& .${tooltipClasses.tooltip}`]: {
    backgroundColor: 'transparent',
  },
}));

export const MobileNavigation = ({
  navigation,
  crnlTokenHook,
  onNavigationToggle,
}: IMobileNavigationProps) => {
  const [showMobileMenu, setShowMobileMenu] = useState(false);

  const [showUserMenu, setShowUserMenu] = useState(false);

  const { isAuthenticated, handleLogin } = useAuth();

  const handleMobileMenuToggle = () => {
    setShowMobileMenu(!showMobileMenu);
  };

  function handleCloseMenu(): void {
    setShowUserMenu(false);
  }

  function handleOpenMenu(): void {
    setShowUserMenu(true);
  }

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
            {isAuthenticated ? (
              <ClickAwayListener onClickAway={handleCloseMenu}>
                <Box>
                  <CustomTooltip
                    title={<UserMenu crnlTokenHook={crnlTokenHook} />}
                    onClose={handleCloseMenu}
                    open={showUserMenu}
                    disableFocusListener
                    disableHoverListener
                    disableTouchListener
                    placement="bottom-start"
                  >
                    <IconButton sx={{ color: 'gray' }} onClick={handleOpenMenu}>
                      <Box display={'flex'} gap={1}>
                        <PersonRoundedIcon fontSize="large" />
                        <KeyboardArrowDownRoundedIcon fontSize="large" />
                      </Box>
                    </IconButton>
                  </CustomTooltip>
                </Box>
              </ClickAwayListener>
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
