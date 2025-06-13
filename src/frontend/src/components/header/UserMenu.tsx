import { Box, CircularProgress, IconButton } from '@mui/material';
import CheckBoxIcon from '@mui/icons-material/CheckBox';
import ContentCopyIcon from '@mui/icons-material/ContentCopy';
import LogoutIcon from '@mui/icons-material/Logout';
import SendRoundedIcon from '@mui/icons-material/SendRounded';
import SyncIcon from '@mui/icons-material/Sync';
import { useState } from 'react';
import { useAuth } from '../../hooks/useAuth';
import { IUseCrnlToken } from '../../hooks/useCrnlToken';

interface IUserMenuProps {
  crnlTokenHook: IUseCrnlToken;
  onCloseMenu: () => void;
  onOpenSendTokenModal: () => void;
}

export const UserMenu = ({
  crnlTokenHook: { balanceData, isLoading: isBalanceLoading, checkBalance },
  onCloseMenu,
  onOpenSendTokenModal,
}: IUserMenuProps) => {
  const [copied, setCopied] = useState(false);

  const { principal, handleLogout } = useAuth();

  function formatPrincipal(principal: string) {
    if (principal.length <= 8) return principal;
    return `${principal.slice(0, 5)}...${principal.slice(-3)}`;
  }

  function handleCopy() {
    if (principal) {
      navigator.clipboard.writeText(principal);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
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
        <Box>{String(balanceData)} CRNL</Box>
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
        <Box>
          <span title={principal || ''} onClick={handleCopy}>
            {principal ? formatPrincipal(principal) : ''}
          </span>
        </Box>
        <Box>
          {!copied ? (
            <IconButton
              sx={{ color: 'lightGray', width: 24, height: 24 }}
              onClick={handleCopy}
            >
              <ContentCopyIcon />
            </IconButton>
          ) : (
            <Box display={'flex'} alignItems={'center'} gap={1}>
              <CheckBoxIcon
                sx={{
                  color: copied ? 'lightGreen' : 'transparent',
                }}
              />
            </Box>
          )}
        </Box>
      </Box>
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
    </Box>
  );
};
