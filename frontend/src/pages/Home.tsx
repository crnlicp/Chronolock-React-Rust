import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import KeyIcon from '@mui/icons-material/Key';
import LockClockIcon from '@mui/icons-material/LockClock';
import MarkEmailReadIcon from '@mui/icons-material/MarkEmailRead';
import {
  Accordion,
  AccordionDetails,
  AccordionSummary,
  Box,
  Button,
  Card,
  CardContent,
  Chip,
  Grid,
  Skeleton,
  Stack,
  Typography,
} from '@mui/material';
import { useMemo } from 'react';
import { Link as RouterLink } from 'react-router';
import { useStats } from '../api/backend';
import { usePaymentInfo } from '../api/ledger';
import { ClockLock } from '../components/clock-lock/ClockLock';
import { formatUnits } from '../lib/format';

const STEPS = [
  {
    icon: <LockClockIcon fontSize="large" />,
    title: 'Pick a moment',
    text: 'Choose when it opens and who can open it: specific people, or everyone.',
  },
  {
    icon: <KeyIcon fontSize="large" />,
    title: 'Seal it in your browser',
    text: 'Your text and files are encrypted before they leave your device. Nobody can read them early, not even us.',
  },
  {
    icon: <MarkEmailReadIcon fontSize="large" />,
    title: 'It opens on time',
    text: 'At the unlock time the Internet Computer releases the key, and only your recipients can open it.',
  },
];

const USES = [
  'Birthday & anniversary surprises',
  'Letters to your future self',
  'Digital time capsules',
  'Inheritance & legacy notes',
  'Timed reveals & announcements',
  'Course material released on schedule',
];

const FAQ = [
  {
    q: 'Can anyone open a chronolock early?',
    a: 'No. Content is encrypted in your browser with a key that only becomes available after the unlock time, through threshold cryptography (vetKeys) run by the Internet Computer. The app, its developers and node operators cannot open it early.',
  },
  {
    q: 'Who can open it after the unlock time?',
    a: 'Only the principals you add as recipients. If you add no recipients, the chronolock is public and anyone signed in can open it after the unlock time. Add yourself if you want to open it too.',
  },
  {
    q: 'What does it cost?',
    a: 'A text chronolock costs 0.2 ICP or 1,000 $CHRONO, a media chronolock 1 ICP or 5,000 $CHRONO, plus small network fees. You pay when you create it.',
  },
  {
    q: 'What is $CHRONO?',
    a: 'The Chronolock community token, launched on the Soon launchpad. It is a standard ICRC token you can hold in any ICP wallet and use to pay for chronolocks.',
  },
  {
    q: 'How do I sign in?',
    a: 'With Internet Identity: passkeys or your Google, Apple or Microsoft account. No passwords. You can top up your Chronolock wallet from OISY.',
  },
  {
    q: 'Can I change or cancel a chronolock?',
    a: 'Its content and unlock time cannot be changed once sealed. As the owner you can burn (delete) it, or transfer it as an NFT (ICRC-7).',
  },
];

function Stat({ label, value }: { label: string; value?: bigint }) {
  return (
    <Box sx={{ textAlign: 'center', minWidth: 120 }}>
      <Typography variant="h4" sx={{ fontWeight: 700 }}>
        {value === undefined ? <Skeleton width={60} sx={{ mx: 'auto' }} /> : value.toLocaleString()}
      </Typography>
      <Typography variant="body2" color="text.secondary">
        {label}
      </Typography>
    </Box>
  );
}

function Pricing() {
  const { data } = usePaymentInfo();
  const p = data?.prices;
  const cards = [
    { title: 'Text chronolock', text: 'A sealed message with title and details.', icp: p?.icp_text, chrono: p?.chrono_text },
    { title: 'Media chronolock', text: 'Add a photo, video or file up to 10 MB.', icp: p?.icp_media, chrono: p?.chrono_media, highlight: true },
  ];
  return (
    <Grid container spacing={3}>
      {cards.map((c) => (
        <Grid key={c.title} size={{ xs: 12, md: 6 }}>
          <Card sx={{ height: '100%', ...(c.highlight && { borderColor: 'primary.main' }) }}>
            <CardContent sx={{ p: 4 }}>
              <Typography variant="h5">{c.title}</Typography>
              <Typography color="text.secondary" sx={{ mb: 3 }}>
                {c.text}
              </Typography>
              <Stack direction="row" spacing={4} sx={{ alignItems: 'baseline' }}>
                <Box>
                  <Typography variant="h3">{c.icp === undefined ? '…' : formatUnits(c.icp, 8)}</Typography>
                  <Typography color="text.secondary">ICP</Typography>
                </Box>
                <Typography color="text.secondary">or</Typography>
                <Box>
                  <Typography variant="h3">{c.chrono === undefined ? '…' : formatUnits(c.chrono, 8)}</Typography>
                  <Typography color="text.secondary">$CHRONO</Typography>
                </Box>
              </Stack>
            </CardContent>
          </Card>
        </Grid>
      ))}
    </Grid>
  );
}

function Section({ title, subtitle, children }: { title: string; subtitle?: string; children: React.ReactNode }) {
  return (
    <Box component="section" sx={{ mt: { xs: 10, md: 14 } }}>
      <Typography variant="h3" component="h2" sx={{ textAlign: 'center', fontSize: { xs: 28, md: 40 } }}>
        {title}
      </Typography>
      {subtitle && (
        <Typography color="text.secondary" sx={{ textAlign: 'center', mt: 1, mb: 5, maxWidth: 620, mx: 'auto' }}>
          {subtitle}
        </Typography>
      )}
      {!subtitle && <Box sx={{ mb: 5 }} />}
      {children}
    </Box>
  );
}

export const HomePage = () => {
  const { data: stats } = useStats();
  // A demo countdown for the hero clock.
  const demoTarget = useMemo(() => new Date(Date.now() + ((3 * 24 + 7) * 3600 + 42 * 60) * 1000), []);

  return (
    <>
      <Grid container spacing={6} sx={{ alignItems: 'center', minHeight: { md: '70vh' } }}>
        <Grid size={{ xs: 12, md: 7 }}>
          <Chip label="Time-lock encryption on the Internet Computer" color="primary" variant="outlined" sx={{ mb: 3 }} />
          <Typography variant="h1" sx={{ fontSize: { xs: 44, md: 68 }, lineHeight: 1.05 }}>
            Send a message{' '}
            <Box component="span" sx={{ background: 'linear-gradient(90deg,#a78bfa,#22d3ee)', backgroundClip: 'text', color: 'transparent' }}>
              to the future.
            </Box>
          </Typography>
          <Typography variant="h6" color="text.secondary" sx={{ mt: 3, fontWeight: 400, maxWidth: 560 }}>
            Seal text, photos and files until the moment you choose. Encrypted in
            your browser and opened only by the people you pick, on time.
          </Typography>
          <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2} sx={{ mt: 5 }}>
            <Button variant="contained" size="large" component={RouterLink} to="/create">
              Create a chronolock
            </Button>
            <Button variant="outlined" size="large" component={RouterLink} to="/collection">
              Explore chronolocks
            </Button>
          </Stack>
        </Grid>
        <Grid size={{ xs: 12, md: 5 }} sx={{ display: 'flex', justifyContent: 'center' }}>
          <ClockLock target={demoTarget} size={420} />
        </Grid>
      </Grid>

      <Card sx={{ mt: 6 }}>
        <CardContent>
          <Stack direction="row" sx={{ justifyContent: 'space-around', flexWrap: 'wrap', gap: 3, py: 1 }}>
            <Stat label="Chronolocks" value={stats?.total_locks} />
            <Stat label="Creators" value={stats?.unique_creators} />
            <Stat label="With media" value={stats?.media_locks} />
            <Stat label="Public" value={stats?.public_locks} />
          </Stack>
        </CardContent>
      </Card>

      <Section title="How it works" subtitle="Three steps, no passwords, no trusted middleman.">
        <Grid container spacing={3}>
          {STEPS.map((s, i) => (
            <Grid key={s.title} size={{ xs: 12, md: 4 }}>
              <Card sx={{ height: '100%' }}>
                <CardContent sx={{ p: 4 }}>
                  <Box sx={{ color: 'primary.light', mb: 2 }}>{s.icon}</Box>
                  <Typography variant="overline" color="text.secondary">
                    Step {i + 1}
                  </Typography>
                  <Typography variant="h5" gutterBottom>
                    {s.title}
                  </Typography>
                  <Typography color="text.secondary">{s.text}</Typography>
                </CardContent>
              </Card>
            </Grid>
          ))}
        </Grid>
      </Section>

      <Section title="Simple pricing" subtitle="Pay once when you create. Use ICP or the $CHRONO community token.">
        <Pricing />
      </Section>

      <Section title="What people seal">
        <Stack direction="row" sx={{ flexWrap: 'wrap', gap: 1.5, justifyContent: 'center' }}>
          {USES.map((u) => (
            <Chip key={u} label={u} sx={{ fontSize: 15, py: 2.5, px: 1 }} />
          ))}
        </Stack>
      </Section>

      <Section title="Questions">
        <Box sx={{ maxWidth: 820, mx: 'auto' }}>
          {FAQ.map((f) => (
            <Accordion key={f.q} disableGutters sx={{ bgcolor: 'transparent', '&:before': { display: 'none' }, borderBottom: 1, borderColor: 'divider' }} elevation={0}>
              <AccordionSummary expandIcon={<ExpandMoreIcon />}>
                <Typography sx={{ fontWeight: 600 }}>{f.q}</Typography>
              </AccordionSummary>
              <AccordionDetails>
                <Typography color="text.secondary">{f.a}</Typography>
              </AccordionDetails>
            </Accordion>
          ))}
        </Box>
      </Section>

      <Card sx={{ mt: 12, textAlign: 'center', borderColor: 'primary.main' }}>
        <CardContent sx={{ py: 6 }}>
          <Typography variant="h4" gutterBottom>
            What will you send to the future?
          </Typography>
          <Button variant="contained" size="large" component={RouterLink} to="/create" sx={{ mt: 2 }}>
            Create a chronolock
          </Button>
        </CardContent>
      </Card>
    </>
  );
};
