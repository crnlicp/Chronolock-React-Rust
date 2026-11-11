import { alpha, createTheme } from '@mui/material/styles';

const violet = '#8b5cf6';
const cyan = '#22d3ee';
const magenta = '#d946ef';
const bg = '#130f2e'; // midnight indigo
const paper = '#1c1742';

/** Ambient "secrecy" backdrop: indigo night with soft glows and faint stars. */
export const ambientBackground = [
  `radial-gradient(900px 600px at 0% 0%, ${alpha(violet, 0.35)}, transparent 60%)`,
  `radial-gradient(800px 520px at 100% 10%, ${alpha(cyan, 0.18)}, transparent 60%)`,
  `radial-gradient(900px 700px at 70% 110%, ${alpha(magenta, 0.2)}, transparent 60%)`,
  'radial-gradient(1.5px 1.5px at 20% 30%, rgba(255,255,255,.35), transparent 50%)',
  'radial-gradient(1px 1px at 70% 20%, rgba(255,255,255,.3), transparent 50%)',
  'radial-gradient(1.5px 1.5px at 85% 65%, rgba(255,255,255,.25), transparent 50%)',
  'radial-gradient(1px 1px at 35% 80%, rgba(255,255,255,.3), transparent 50%)',
  'radial-gradient(1px 1px at 55% 45%, rgba(255,255,255,.2), transparent 50%)',
  `linear-gradient(160deg, #1b1446 0%, ${bg} 45%, #0e1433 100%)`,
].join(', ');

export const theme = createTheme({
  palette: {
    mode: 'dark',
    primary: { main: violet, light: '#a78bfa', dark: '#6d28d9', contrastText: '#fff' },
    secondary: { main: cyan, contrastText: '#06202a' },
    success: { main: '#34d399' },
    warning: { main: '#fbbf24' },
    error: { main: '#f87171' },
    background: { default: bg, paper },
    text: { primary: '#f1eefe', secondary: '#b3acd6' },
    divider: alpha('#ffffff', 0.08),
  },
  shape: { borderRadius: 14 },
  typography: {
    fontFamily: '"Inter", system-ui, -apple-system, "Segoe UI", sans-serif',
    h1: { fontFamily: '"Space Grotesk", "Inter", sans-serif', fontWeight: 700, letterSpacing: '-0.02em' },
    h2: { fontFamily: '"Space Grotesk", "Inter", sans-serif', fontWeight: 700, letterSpacing: '-0.02em' },
    h3: { fontFamily: '"Space Grotesk", "Inter", sans-serif', fontWeight: 700 },
    h4: { fontFamily: '"Space Grotesk", "Inter", sans-serif', fontWeight: 600 },
    h5: { fontFamily: '"Space Grotesk", "Inter", sans-serif', fontWeight: 600 },
    h6: { fontWeight: 600 },
    button: { textTransform: 'none', fontWeight: 600 },
    overline: { letterSpacing: '0.12em', fontWeight: 600 },
  },
  components: {
    MuiCssBaseline: {
      styleOverrides: {
        body: {
          backgroundColor: bg,
          backgroundImage: ambientBackground,
          backgroundAttachment: 'fixed',
          minHeight: '100vh',
        },
        a: { color: 'inherit' },
      },
    },
    MuiButton: {
      defaultProps: { disableElevation: true },
      styleOverrides: {
        root: {
          borderRadius: 999,
          paddingInline: 20,
          variants: [
            {
              props: { variant: 'contained', color: 'primary' },
              style: { backgroundImage: `linear-gradient(135deg, ${violet}, #6366f1)` },
            },
          ],
        },
      },
    },
    MuiPaper: {
      styleOverrides: {
        root: { backgroundImage: 'none' },
        outlined: { borderColor: alpha('#ffffff', 0.08) },
      },
    },
    MuiCard: {
      defaultProps: { variant: 'outlined' },
      styleOverrides: {
        root: {
          background: alpha('#ffffff', 0.045),
          backdropFilter: 'blur(10px)',
          borderColor: alpha('#c4b5fd', 0.14),
        },
      },
    },
    MuiDialog: {
      styleOverrides: { paper: { border: `1px solid ${alpha('#ffffff', 0.08)}` } },
    },
    MuiTextField: { defaultProps: { fullWidth: true } },
    MuiChip: { styleOverrides: { root: { fontWeight: 600 } } },
    MuiTab: { styleOverrides: { root: { textTransform: 'none', fontWeight: 600 } } },
  },
});
