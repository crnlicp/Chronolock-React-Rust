import { Fragment, useState } from 'react';
import { PreLoader } from '../PreLoader';
import { DrawerNavigation } from './DrawerNavigation';
import { MobileNavigation } from './MobileNavigation';
import { MainNavigation } from './MainNavigation';
import { useCrnlToken } from '../../hooks/useCrnlToken';
import { Backdrop, CircularProgress } from '@mui/material';
import { useAuth } from '../../hooks/useAuth';
import { SendTokenModal } from './SendTokenModal';

export const Header = () => {
  const crnlTokenHook = useCrnlToken();
  const { isAuthLoading, principal } = useAuth();

  const [showSendTokenModal, setShowSendTokenModal] = useState(false);
  const [navigation, setNavigation] = useState(false);

  const handleNavigationToggle = (value: boolean) => {
    setNavigation(value);
  };

  const handleOpenSendTokenModal = () => {
    setShowSendTokenModal(true);
  };

  const handleCloseSendTokenModal = () => {
    setShowSendTokenModal(false);
  };

  return (
    <Fragment>
      <PreLoader />
      <DrawerNavigation
        navigation={navigation}
        onNavigationToggle={handleNavigationToggle}
      />
      <MobileNavigation
        navigation={navigation}
        crnlTokenHook={crnlTokenHook}
        onNavigationToggle={handleNavigationToggle}
        onOpenSendTokenModal={handleOpenSendTokenModal}
      />
      <MainNavigation
        crnlTokenHook={crnlTokenHook}
        onNavigationToggle={handleNavigationToggle}
        onOpenSendTokenModal={handleOpenSendTokenModal}
      />
      <Backdrop
        sx={(theme) => ({ color: '#fff', zIndex: theme.zIndex.drawer + 1 })}
        open={isAuthLoading || false}
      >
        <CircularProgress size={70} />
      </Backdrop>
      <SendTokenModal
        open={showSendTokenModal}
        onClose={handleCloseSendTokenModal}
        crnlTokenHook={crnlTokenHook}
        principal={principal}
      />
    </Fragment>
  );
};
