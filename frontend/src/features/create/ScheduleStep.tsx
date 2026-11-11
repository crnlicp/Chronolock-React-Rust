import {
  Alert,
  Autocomplete,
  Box,
  Button,
  Checkbox,
  Chip,
  FormControlLabel,
  Stack,
  TextField,
  ToggleButton,
  ToggleButtonGroup,
  Typography,
} from '@mui/material';
import { DateTimePicker } from '@mui/x-date-pickers';
import { Principal } from '@icp-sdk/core/principal';
import dayjs, { type Dayjs } from 'dayjs';
import { useState } from 'react';
import type { Audience, Draft } from './types';

const PRESETS: [string, number, dayjs.ManipulateType][] = [
  ['1 hour', 1, 'hour'],
  ['1 day', 1, 'day'],
  ['1 week', 1, 'week'],
  ['1 month', 1, 'month'],
  ['1 year', 1, 'year'],
  ['10 years', 10, 'year'],
];

function isPrincipal(text: string): boolean {
  try {
    const p = Principal.fromText(text.trim());
    return !p.isAnonymous() && p.toText() === text.trim();
  } catch {
    return false;
  }
}

interface Props {
  draft: Draft;
  maxRecipients: number;
  onChange: (patch: Partial<Draft>) => void;
  onNext: () => void;
}

export const ScheduleStep = ({ draft, maxRecipients, onChange, onNext }: Props) => {
  const [input, setInput] = useState('');
  const [inputError, setInputError] = useState<string | null>(null);
  const value: Dayjs | null = draft.unlockAt ? dayjs.unix(draft.unlockAt) : null;
  const inFuture = !!draft.unlockAt && draft.unlockAt > Date.now() / 1000 + 30;
  const peopleOk = draft.audience !== 'people' || draft.people.length > 0;

  const addPeople = (texts: string[]) => {
    const parts = texts.flatMap((t) => t.split(/[\s,;]+/)).map((t) => t.trim()).filter(Boolean);
    const bad = parts.filter((p) => !isPrincipal(p));
    if (bad.length) {
      setInputError(`Not a valid principal: ${bad[0]}`);
      return;
    }
    const next = [...new Set([...draft.people, ...parts])].slice(0, maxRecipients);
    onChange({ people: next });
    setInput('');
    setInputError(null);
  };

  return (
    <Stack spacing={4}>
      <Box>
        <Typography variant="h6" gutterBottom>
          When should it open?
        </Typography>
        <Stack direction="row" sx={{ flexWrap: 'wrap', gap: 1, mb: 2 }}>
          {PRESETS.map(([label, n, unit]) => (
            <Chip
              key={label}
              label={`In ${label}`}
              onClick={() => onChange({ unlockAt: dayjs().add(n, unit).second(0).unix() })}
              variant="outlined"
            />
          ))}
        </Stack>
        <DateTimePicker
          label="Unlock date and time"
          value={value}
          disablePast
          ampm={false}
          onChange={(v) => onChange({ unlockAt: v && v.isValid() ? v.unix() : null })}
          slotProps={{ textField: { fullWidth: true } }}
        />
        {draft.unlockAt && !inFuture && (
          <Alert severity="warning" sx={{ mt: 1 }}>
            Pick a time in the future.
          </Alert>
        )}
      </Box>

      <Box>
        <Typography variant="h6" gutterBottom>
          Who can open it?
        </Typography>
        <ToggleButtonGroup
          exclusive
          value={draft.audience}
          onChange={(_, v: Audience | null) => v && onChange({ audience: v })}
          sx={{ flexWrap: 'wrap', mb: 2 }}
        >
          <ToggleButton value="me">Only me</ToggleButton>
          <ToggleButton value="people">Specific people</ToggleButton>
          <ToggleButton value="public">Everyone</ToggleButton>
        </ToggleButtonGroup>
        {draft.audience === 'public' && (
          <Typography color="text.secondary">
            Anyone signed in can open it after the unlock time.
          </Typography>
        )}
        {draft.audience === 'me' && (
          <Typography color="text.secondary">A letter to your future self.</Typography>
        )}
        {draft.audience === 'people' && (
          <>
            <Autocomplete
              multiple
              freeSolo
              options={[]}
              value={draft.people}
              inputValue={input}
              onInputChange={(_, v) => {
                setInput(v);
                setInputError(null);
              }}
              onChange={(_, v) => {
                const added = (v as string[]).filter((x) => !draft.people.includes(x));
                if (added.length) addPeople(added);
                else onChange({ people: v as string[] });
              }}
              onBlur={() => input.trim() && addPeople([input])}
              renderValue={(vals, getItemProps) =>
                vals.map((p, i) => {
                  const { key, ...rest } = getItemProps({ index: i });
                  return <Chip key={key} label={`${p.slice(0, 11)}…${p.slice(-3)}`} {...rest} />;
                })
              }
              renderInput={(params) => (
                <TextField
                  {...params}
                  label="Recipient principals"
                  placeholder="Paste a principal and press Enter"
                  error={!!inputError}
                  helperText={inputError ?? `Up to ${maxRecipients}. Recipients can find their principal on their Wallet page.`}
                />
              )}
            />
            <FormControlLabel
              sx={{ mt: 1 }}
              control={<Checkbox checked={draft.includeMe} onChange={(e) => onChange({ includeMe: e.target.checked })} />}
              label="I can open it too"
            />
          </>
        )}
      </Box>

      <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
        <Button variant="contained" size="large" disabled={!inFuture || !peopleOk} onClick={onNext}>
          Next
        </Button>
      </Box>
    </Stack>
  );
};
