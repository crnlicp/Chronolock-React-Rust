import { Fragment, useState } from 'react';
import { PreLoader } from '../PreLoader';
import { DrawerNavigation } from './DrawerNavigation';
import { MobileNavigation } from './MobileNavigation';
import { MainNavigation } from './MainNavigation';

export const Header = () => {
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
        onNavigationToggle={handleNavigationToggle}
      />
      <MainNavigation onNavigationToggle={handleNavigationToggle} />
    </Fragment>
  );
};
