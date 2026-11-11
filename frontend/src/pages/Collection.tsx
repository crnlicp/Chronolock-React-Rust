import { Box, Button, Tab, Tabs } from '@mui/material';
import { Link as RouterLink, useSearchParams } from 'react-router';
import type { ListKind } from '../api/backend';
import { useAuth } from '../auth/AuthProvider';
import { PageHeader } from '../components/ui/PageHeader';
import { SignInPrompt } from '../components/ui/SignInPrompt';
import { LockList } from '../features/collection/LockList';

const TABS: { id: ListKind; label: string; empty: string; auth: boolean }[] = [
  { id: 'unlockable', label: 'Ready to open', empty: 'Nothing to open yet. Chronolocks shared with you appear here once they unlock.', auth: true },
  { id: 'received', label: 'Shared with me', empty: 'No chronolocks have been shared with you yet.', auth: true },
  { id: 'owned', label: 'Created by me', empty: "You haven't created any chronolocks yet.", auth: true },
  { id: 'all', label: 'Explore', empty: 'No chronolocks yet. Be the first!', auth: false },
];

export const CollectionPage = () => {
  const { isAuthenticated } = useAuth();
  const [params, setParams] = useSearchParams();
  const current = TABS.find((t) => t.id === params.get('tab')) ?? (isAuthenticated ? TABS[0] : TABS[3]);

  return (
    <>
      <PageHeader
        title="Collection"
        subtitle="Chronolocks shared with you, the ones you created, and everything sealed on Chronolock."
        action={
          <Button variant="contained" component={RouterLink} to="/create">
            New chronolock
          </Button>
        }
      />
      <Box sx={{ borderBottom: 1, borderColor: 'divider', mb: 4 }}>
        <Tabs
          value={current.id}
          onChange={(_, v) => setParams({ tab: v })}
          variant="scrollable"
          allowScrollButtonsMobile
        >
          {TABS.map((t) => (
            <Tab key={t.id} value={t.id} label={t.label} />
          ))}
        </Tabs>
      </Box>
      {current.auth && !isAuthenticated ? (
        <SignInPrompt message="Sign in to see chronolocks shared with you and the ones you created." />
      ) : (
        <LockList key={current.id} kind={current.id} empty={current.empty} />
      )}
    </>
  );
};
