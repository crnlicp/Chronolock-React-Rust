import CheckCircleIcon from '@mui/icons-material/CheckCircle';
import { Box, Button, Card, CardContent, Stack, Step, StepLabel, Stepper, Typography } from '@mui/material';
import { useEffect, useState } from 'react';
import { Link as RouterLink } from 'react-router';
import { useBackendConfig } from '../api/backend';
import { useAuth } from '../auth/AuthProvider';
import { CopyField } from '../components/ui/CopyField';
import { PageHeader } from '../components/ui/PageHeader';
import { SignInPrompt } from '../components/ui/SignInPrompt';
import { ContentStep } from '../features/create/ContentStep';
import { ReviewStep } from '../features/create/ReviewStep';
import { ScheduleStep } from '../features/create/ScheduleStep';
import { emptyDraft, type Draft } from '../features/create/types';
import { generateAesKey } from '../lib/crypto';

const STEPS = ['When & who', 'Content', 'Review & pay'];

export const CreatePage = () => {
  const { isAuthenticated } = useAuth();
  const { data: config } = useBackendConfig();
  const [step, setStep] = useState(0);
  const [draft, setDraft] = useState<Draft>(emptyDraft);
  const [aesKey, setAesKey] = useState<CryptoKey | null>(null);
  const [created, setCreated] = useState<bigint | null>(null);

  useEffect(() => {
    void generateAesKey().then(setAesKey);
  }, []);

  if (!isAuthenticated) return <SignInPrompt message="Sign in with Internet Identity to create a chronolock." />;

  const patch = (p: Partial<Draft>) => setDraft((d) => ({ ...d, ...p }));
  const limits = config?.limits;

  if (created !== null) {
    const url = `${window.location.origin}/chronolock/${created}`;
    return (
      <Card sx={{ maxWidth: 560, mx: 'auto', textAlign: 'center' }}>
        <CardContent sx={{ p: 5 }}>
          <CheckCircleIcon color="success" sx={{ fontSize: 64 }} />
          <Typography variant="h4" gutterBottom>
            Sealed!
          </Typography>
          <Typography color="text.secondary" sx={{ mb: 3 }}>
            Chronolock #{created.toString()} is locked until its unlock time. Share the link with
            your recipients.
          </Typography>
          <Box sx={{ textAlign: 'left' }}>
            <CopyField label="Share link" value={url} />
          </Box>
          <Stack direction="row" spacing={2} sx={{ justifyContent: 'center', mt: 2 }}>
            <Button variant="contained" component={RouterLink} to={`/chronolock/${created}`}>
              View chronolock
            </Button>
            <Button
              onClick={() => {
                setDraft(emptyDraft);
                setCreated(null);
                setStep(0);
                void generateAesKey().then(setAesKey);
              }}
            >
              Create another
            </Button>
          </Stack>
        </CardContent>
      </Card>
    );
  }

  return (
    <>
      <PageHeader title="New chronolock" subtitle="Everything is encrypted in your browser before it leaves your device." />
      <Stepper activeStep={step} alternativeLabel sx={{ mb: 5 }}>
        {STEPS.map((s) => (
          <Step key={s}>
            <StepLabel>{s}</StepLabel>
          </Step>
        ))}
      </Stepper>
      <Card>
        <CardContent sx={{ p: { xs: 2.5, md: 5 } }}>
          {step === 0 && (
            <ScheduleStep
              draft={draft}
              maxRecipients={(limits?.max_recipients ?? 50) - 1}
              onChange={patch}
              onNext={() => setStep(1)}
            />
          )}
          {step === 1 && (
            <ContentStep
              draft={draft}
              aesKey={aesKey}
              maxMediaBytes={Number(limits?.max_media_bytes ?? 10_485_760n)}
              chunkBytes={Number(limits?.max_chunk_bytes ?? 1_900_000n)}
              onChange={patch}
              onBack={() => setStep(0)}
              onNext={() => setStep(2)}
            />
          )}
          {step === 2 && aesKey && (
            <ReviewStep draft={draft} aesKey={aesKey} onBack={() => setStep(1)} onCreated={setCreated} />
          )}
        </CardContent>
      </Card>
    </>
  );
};
