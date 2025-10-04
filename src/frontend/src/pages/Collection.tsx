import React, { useEffect, useState } from 'react';
import { Box, Tab, Tabs } from '@mui/material';
import { AllChronolocks } from '../components/collection/AllChronolocks';
import { MyChronolocks } from '../components/collection/MyChronolocks';
import { DecryptableChronolocks } from '../components/collection/DecryptableChronolocks';

interface TabPanelProps {
  children?: React.ReactNode;
  index: number;
  value: number;
}

function TabPanel(props: TabPanelProps) {
  const { children, value, index, ...other } = props;

  return (
    <div
      role="tabpanel"
      hidden={value !== index}
      id={`collection-tabpanel-${index}`}
      aria-labelledby={`collection-tab-${index}`}
      {...other}
    >
      {value === index && <Box sx={{ p: 3 }}>{children}</Box>}
    </div>
  );
}

function a11yProps(index: number) {
  return {
    id: `collection-tab-${index}`,
    'aria-controls': `collection-tabpanel-${index}`,
    sx: { color: 'white' },
  };
}

export const Collection = () => {
  const [value, setValue] = useState(0);

  const handleChange = (_event: React.SyntheticEvent, newValue: number) => {
    setValue(newValue);
  };

  useEffect(() => {
    window.scrollTo({ top: 0, behavior: 'smooth' });
  }, []);

  return (
    <div className="container page_container">
      <Box sx={{ width: '100%' }} mt={4}>
        <Box sx={{ borderBottom: 1, borderColor: 'divider', mb: 3 }}>
          <Tabs
            value={value}
            onChange={handleChange}
            aria-label="collection tabs"
          >
            <Tab label="Chronolocks" {...a11yProps(0)} />
            <Tab label="My Chronolocks" {...a11yProps(1)} />
            <Tab label="Encrypted for you" {...a11yProps(2)} />
          </Tabs>
        </Box>

        <TabPanel value={value} index={0}>
          <AllChronolocks />
        </TabPanel>

        <TabPanel value={value} index={1}>
          <MyChronolocks />
        </TabPanel>

        <TabPanel value={value} index={2}>
          <DecryptableChronolocks />
        </TabPanel>
      </Box>
    </div>
  );
};
