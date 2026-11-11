import AccountBalanceWalletIcon from '@mui/icons-material/AccountBalanceWallet';
import AddIcon from '@mui/icons-material/Add';
import CloseIcon from '@mui/icons-material/Close';
import LogoutIcon from '@mui/icons-material/Logout';
import MenuIcon from '@mui/icons-material/Menu';
import {
  AppBar,
  Avatar,
  Box,
  Button,
  Container,
  Divider,
  Drawer,
  IconButton,
  List,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Menu,
  MenuItem,
  Toolbar,
  Typography,
} from '@mui/material';
import { alpha } from '@mui/material/styles';
import { useState } from 'react';
import { NavLink, Link as RouterLink } from 'react-router';
import { useBalance, usePaymentInfo, useTokenMeta } from '../../api/ledger';
import { useAuth } from '../../auth/AuthProvider';
import { formatUnits } from '../../lib/format';
import { CopyField, shortPrincipal } from '../ui/CopyField';

const NAV = [
  { to: '/', label: 'Home', end: true },
  { to: '/create', label: 'Create' },
  { to: '/collection', label: 'Collection' },
  { to: '/wallet', label: 'Wallet' },
];

function IcpBalance() {
  const { data: info } = usePaymentInfo();
  const { data: meta } = useTokenMeta(info?.icp_ledger);
  const { data: balance } = useBalance(info?.icp_ledger);
  if (!meta || balance === undefined) return null;
  return (
    <Typography variant="body2" color="text.secondary" sx={{ display: { xs: 'none', sm: 'block' } }}>
      {formatUnits(balance, meta.decimals, 2)} ICP
    </Typography>
  );
}

function AccountMenu() {
  const { principal, signOut } = useAuth();
  const [anchor, setAnchor] = useState<HTMLElement | null>(null);
  if (!principal) return null;
  return (
    <>
      <Button
        onClick={(e) => setAnchor(e.currentTarget)}
        variant="outlined"
        color="inherit"
        sx={{ borderColor: 'divider', gap: 1.5, pl: 1 }}
        aria-label="Account menu"
      >
        <Avatar sx={{ width: 26, height: 26, bgcolor: 'primary.main', fontSize: 13 }}>
          {principal.slice(0, 2).toUpperCase()}
        </Avatar>
        <IcpBalance />
        <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
          {shortPrincipal(principal)}
        </Typography>
      </Button>
      <Menu
        anchorEl={anchor}
        open={!!anchor}
        onClose={() => setAnchor(null)}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
        transformOrigin={{ vertical: 'top', horizontal: 'right' }}
        slotProps={{ paper: { sx: { width: 320, mt: 1 } } }}
      >
        <Box sx={{ px: 2, pt: 1 }}>
          <CopyField label="Your principal" value={principal} />
        </Box>
        <Divider />
        <MenuItem component={RouterLink} to="/wallet" onClick={() => setAnchor(null)}>
          <ListItemIcon>
            <AccountBalanceWalletIcon fontSize="small" />
          </ListItemIcon>
          Wallet
        </MenuItem>
        <MenuItem component={RouterLink} to="/create" onClick={() => setAnchor(null)}>
          <ListItemIcon>
            <AddIcon fontSize="small" />
          </ListItemIcon>
          New chronolock
        </MenuItem>
        <MenuItem
          onClick={() => {
            setAnchor(null);
            void signOut();
          }}
        >
          <ListItemIcon>
            <LogoutIcon fontSize="small" />
          </ListItemIcon>
          Sign out
        </MenuItem>
      </Menu>
    </>
  );
}

export const Header = () => {
  const { isAuthenticated, signIn, isLoading } = useAuth();
  const [drawer, setDrawer] = useState(false);

  return (
    <AppBar
      position="sticky"
      elevation={0}
      sx={{
        bgcolor: (t) => alpha(t.palette.background.default, 0.7),
        backdropFilter: 'blur(14px)',
        borderBottom: 1,
        borderColor: 'divider',
      }}
    >
      <Container maxWidth="lg">
        <Toolbar disableGutters sx={{ gap: 2 }}>
          <Box
            component={RouterLink}
            to="/"
            sx={{ display: 'flex', alignItems: 'center', gap: 1, textDecoration: 'none', mr: 2 }}
          >
            <Box component="img" src="/assets/brand/logo-mark.svg" alt="" sx={{ width: 30, height: 35 }} />
            <Typography variant="h6" sx={{ fontFamily: '"Space Grotesk", sans-serif' }}>
              Chronolock
            </Typography>
          </Box>
          <Box sx={{ display: { xs: 'none', md: 'flex' }, gap: 0.5, flexGrow: 1 }}>
            {NAV.map((n) => (
              <Button
                key={n.to}
                component={NavLink}
                to={n.to}
                end={n.end}
                color="inherit"
                sx={{ color: 'text.secondary', '&.active': { color: 'text.primary', bgcolor: 'action.selected' } }}
              >
                {n.label}
              </Button>
            ))}
          </Box>
          <Box sx={{ flexGrow: { xs: 1, md: 0 } }} />
          {isAuthenticated ? (
            <AccountMenu />
          ) : (
            <Button variant="contained" onClick={() => void signIn()} disabled={isLoading}>
              Sign in
            </Button>
          )}
          <IconButton
            sx={{ display: { md: 'none' } }}
            onClick={() => setDrawer(true)}
            aria-label="Open menu"
          >
            <MenuIcon />
          </IconButton>
        </Toolbar>
      </Container>
      <Drawer anchor="right" open={drawer} onClose={() => setDrawer(false)}>
        <Box sx={{ width: 260, p: 1 }}>
          <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
            <IconButton onClick={() => setDrawer(false)} aria-label="Close menu">
              <CloseIcon />
            </IconButton>
          </Box>
          <List>
            {NAV.map((n) => (
              <ListItemButton
                key={n.to}
                component={NavLink}
                to={n.to}
                end={n.end}
                onClick={() => setDrawer(false)}
                sx={{ borderRadius: 2, '&.active': { bgcolor: 'action.selected' } }}
              >
                <ListItemText primary={n.label} />
              </ListItemButton>
            ))}
          </List>
        </Box>
      </Drawer>
    </AppBar>
  );
};
