import { Box, CircularProgress, IconButton } from '@mui/material';
import CheckBoxIcon from '@mui/icons-material/CheckBox';
import ContentCopyIcon from '@mui/icons-material/ContentCopy';
import LogoutIcon from '@mui/icons-material/Logout';
import LockClockIcon from '@mui/icons-material/LockClock';
import SendRoundedIcon from '@mui/icons-material/SendRounded';
import SyncIcon from '@mui/icons-material/Sync';
import { useState } from 'react';
import { useAuth } from '../../hooks/useAuth';
import { IUseCrnlToken } from '../../hooks/useCrnlToken';
import { NavLink } from 'react-router';

interface IUserMenuProps {
  crnlTokenHook: IUseCrnlToken;
  onCloseMenu: () => void;
  onOpenSendTokenModal: () => void;
}

export const UserMenu = ({
  crnlTokenHook: {
    balanceData,
    isLoading: isBalanceLoading,
    referralCode,
    checkBalance,
  },
  onCloseMenu,
  onOpenSendTokenModal,
}: IUserMenuProps) => {
  const { principal, handleLogout } = useAuth();

  const [copiedWallet, setCopiedWallet] = useState(false);
  const [copiedURL, setCopiedURL] = useState(false);

  const referralLink =
    process.env.DFX_NETWORK === 'ic'
      ? `https://${process.env.CANISTER_ID_FRONTEND}.icp0.io/?referral_code=${referralCode}`
      : `http://${process.env.CANISTER_ID_FRONTEND}.localhost:4943/?referral_code=${referralCode}`;

  function formatPrincipal(principal: string) {
    if (principal.length <= 8) return principal;
    return `${principal.slice(0, 5)}...${principal.slice(-3)}`;
  }

  function handleCopyWallet() {
    if (principal) {
      navigator.clipboard.writeText(principal);
      setCopiedWallet(true);
      setTimeout(() => setCopiedWallet(false), 2000);
    }
  }

  function handleCopyURL() {
    if (referralLink) {
      navigator.clipboard.writeText(referralLink);
      setCopiedURL(true);
      setTimeout(() => setCopiedURL(false), 2000);
    }
  }

  function handleChekcBalance() {
    if (principal) {
      checkBalance();
    }
  }

  function handleOpenSendTokenModal() {
    onCloseMenu();
    onOpenSendTokenModal();
  }

  function handleClickCreate() {
    onCloseMenu();
  }

  return (
    <Box
      sx={{
        color: 'lightGray',
        boxShadow: 3,
        borderRadius: 3,
        padding: 2,
        width: 300,
        height: 'auto',
        display: 'flex',
        flexDirection: 'column',
        zIndex: 0,
        fontSize: '1.6em',
        background:
          'linear-gradient(135deg,rgb(118, 42, 95) 0%,rgb(70, 3, 101) 100%, rgb(118, 42, 95) 100%)',
      }}
      gap={2}
    >
      <Box
        display={'flex'}
        alignItems={'center'}
        sx={{
          fontFamily: 'monospace',
          fontSize: '0.8em',
        }}
        gap={2}
      >
        <Box width={'100%'}>Your Credit: {String(balanceData)} CRNL</Box>
        <Box width={24} height={24}>
          {isBalanceLoading ? (
            <CircularProgress size={24} />
          ) : (
            <IconButton
              sx={{ color: 'lightGray', width: 24, height: 24 }}
              onClick={handleChekcBalance}
            >
              <SyncIcon />
            </IconButton>
          )}
        </Box>
      </Box>
      <Box
        display={'flex'}
        alignItems={'center'}
        gap={2}
        sx={{ fontFamily: 'monospace', fontSize: '0.8em' }}
      >
        <Box width={'100%'}>
          <span title={principal || ''} onClick={handleCopyWallet}>
            Principal ID: {principal ? formatPrincipal(principal) : ''}
          </span>
        </Box>
        <Box>
          {!copiedWallet ? (
            <IconButton
              sx={{ color: 'lightGray', width: 24, height: 24 }}
              onClick={handleCopyWallet}
            >
              <ContentCopyIcon />
            </IconButton>
          ) : (
            <Box display={'flex'} alignItems={'center'} gap={1}>
              <CheckBoxIcon
                sx={{
                  color: copiedWallet ? 'lightGreen' : 'transparent',
                }}
              />
            </Box>
          )}
        </Box>
      </Box>
      {referralCode ? (
        <Box
          display={'flex'}
          alignItems={'center'}
          gap={2}
          sx={{ fontFamily: 'monospace', fontSize: '0.8em' }}
        >
          <Box width={'100%'}>
            <span title={principal || ''} onClick={handleCopyURL}>
              Referral Link:
            </span>
          </Box>
          <Box>
            {!copiedURL ? (
              <IconButton
                sx={{ color: 'lightGray', width: 24, height: 24 }}
                onClick={handleCopyURL}
              >
                <ContentCopyIcon />
              </IconButton>
            ) : (
              <Box display={'flex'} alignItems={'center'} gap={1}>
                <CheckBoxIcon
                  sx={{
                    color: copiedURL ? 'lightGreen' : 'transparent',
                  }}
                />
              </Box>
            )}
          </Box>
        </Box>
      ) : null}
      <Box>
        <button
          onClick={handleOpenSendTokenModal}
          className="metaportal_fn_button"
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            backgroundColor: 'transparent',
            fontSize: '0.9em',
            padding: '0.3em 0.8em',
            border: 'none',
            width: '100%',
            cursor: 'pointer',
          }}
        >
          <Box mx={2}>Send Token</Box>
          <SendRoundedIcon />
        </button>
      </Box>
      <Box>
        <button
          onClick={handleLogout}
          className="metaportal_fn_button"
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            backgroundColor: 'transparent',
            fontSize: '0.9em',
            padding: '0.3em 0.8em',
            border: 'none',
            width: '100%',
            cursor: 'pointer',
          }}
        >
          <Box mx={2}>Logout</Box>
          <LogoutIcon />
        </button>
      </Box>
      <Box>
        <NavLink
          to="/Create"
          className="metaportal_fn_button "
          onClick={handleClickCreate}
        >
          <Box mx={2} display={'flex'} alignItems={'center'}>
            Create Chronolock
            <LockClockIcon sx={{ ml: 1 }} />
          </Box>
        </NavLink>
      </Box>
    </Box>
  );
};
