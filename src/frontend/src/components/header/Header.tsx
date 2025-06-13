import { Fragment, useState } from 'react';
import { PreLoader } from '../PreLoader';
import { DrawerNavigation } from './DrawerNavigation';
import { MobileNavigation } from './MobileNavigation';
import { MainNavigation } from './MainNavigation';
import { useCrnlToken } from '../../hooks/useCrnlToken';
import { Backdrop, CircularProgress } from '@mui/material';
import { useAuth } from '../../hooks/useAuth';

export const Header = () => {
  const crnlTokenHook = useCrnlToken();
  const { isAuthLoading } = useAuth();

  const [navigation, setNavigation] = useState(false);

  const handleNavigationToggle = (value: boolean) => {
    setNavigation(value);
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
      />
      <MainNavigation
        crnlTokenHook={crnlTokenHook}
        onNavigationToggle={handleNavigationToggle}
      />
      <Backdrop
        sx={(theme) => ({ color: '#fff', zIndex: theme.zIndex.drawer + 1 })}
        open={isAuthLoading || false}
      >
        <CircularProgress size={70} />
      </Backdrop>
    </Fragment>
  );
};
