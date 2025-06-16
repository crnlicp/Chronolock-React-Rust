import { useEffect, useState } from 'react';
import {
  Box,
  ClickAwayListener,
  IconButton,
  styled,
  Tooltip,
  tooltipClasses,
  TooltipProps,
} from '@mui/material';
import PersonRoundedIcon from '@mui/icons-material/PersonRounded';
import KeyboardArrowDownRoundedIcon from '@mui/icons-material/KeyboardArrowDownRounded';
import { NavLink } from 'react-router';
import { stickyNav } from '../../utils/utility';
import { useAuth } from '../../hooks/useAuth';
import { useMenuClick } from '../../hooks/useMenuClick';
import { UserMenu } from './UserMenu';
import { IUseCrnlToken } from '../../hooks/useCrnlToken';

interface IHeaderProps {
  crnlTokenHook: IUseCrnlToken;
  onNavigationToggle: (value: boolean) => void;
  onOpenSendTokenModal: () => void;
}

const CustomTooltip = styled(({ className, ...props }: TooltipProps) => (
  <Tooltip {...props} classes={{ popper: className }} />
))(() => ({
  [`& .${tooltipClasses.tooltip}`]: {
    backgroundColor: 'transparent',
  },
}));

export const MainNavigation = ({
  crnlTokenHook,
  onNavigationToggle,
  onOpenSendTokenModal,
}: IHeaderProps) => {
  const { isAuthenticated, handleLogin } = useAuth();

  const [showMenu, setShowMenu] = useState(false);

  useEffect(() => {
    stickyNav();
  }, []);

  useMenuClick();

  function handleCloseMenu(): void {
    setShowMenu(false);
  }

  function handleOpenMenu(): void {
    setShowMenu(true);
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
                <NavLink to="/#home" className="creative_link">
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
              <ClickAwayListener onClickAway={handleCloseMenu}>
                <Box>
                  <CustomTooltip
                    title={
                      <UserMenu
                        crnlTokenHook={crnlTokenHook}
                        onCloseMenu={handleCloseMenu}
                        onOpenSendTokenModal={onOpenSendTokenModal}
                      />
                    }
                    onClose={handleCloseMenu}
                    open={showMenu}
                    disableFocusListener
                    disableHoverListener
                    disableTouchListener
                    placement="bottom-start"
                    id="user-menu-tooltip-desktop"
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
      </div>
    </header>
  );
};
