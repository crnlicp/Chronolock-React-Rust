import { Box, CircularProgress, Dialog, Typography } from '@mui/material';
import { useState } from 'react';
import { useCrnlToken } from '../../hooks/useCrnlToken';
import { Principal } from '@dfinity/principal';

interface ISendTokenModalProps {
  open: boolean;
  onClose: () => void;
}

export const SendTokenModal = ({ open, onClose }: ISendTokenModalProps) => {
  const [address, setAddress] = useState('');
  const [amount, setAmount] = useState('');

  const { transfer, isLoading, transferData } = useCrnlToken();

  const handleChaneAddress = (e: React.ChangeEvent<HTMLInputElement>) => {
    setAddress(e.target.value);
  };

  const handleChangeAmount = (e: React.ChangeEvent<HTMLInputElement>) => {
    setAmount(e.target.value);
  };

  const handleSendToken = async () => {
    if (!address || !amount) {
      console.error('Address and amount are required', { address, amount });
      return;
    }
    const parsedAmount = parseFloat(amount);
    if (isNaN(parsedAmount) || parsedAmount <= 0) {
      console.error('Invalid amount');
      return;
    }
    // Convert to smallest unit and string for Nat
    const natAmount = (parsedAmount * 1e8).toLocaleString('fullwide', {
      useGrouping: false,
    });

    try {
      console.log('Sending token to:', address, 'Amount:', natAmount);
      const parsedAmount = parseFloat(amount);
      await transfer({
        to: Principal.fromText(address),
        amount: BigInt(Math.round(parsedAmount * 1e8)),
      });
      console.log('Transfer successful!', transferData);
      // setAddress('');
      // setAmount('');
      // onClose();
    } catch (error) {
      console.error('Error sending token:', error);
    }
  };

  return (
    <Dialog fullWidth open={open} onClose={onClose}>
      <Box
        display={'flex'}
        flexDirection="column"
        p={2}
        gap={3}
        sx={{
          background:
            'linear-gradient(135deg,rgb(164, 26, 122) 0%,rgb(81, 9, 114) 100%, rgb(118, 42, 95) 100%)',
        }}
      >
        <Box>
          <Typography color="lightGray" fontWeight={'bold'} variant="h4">
            Send Token
          </Typography>
        </Box>
        <Box>
          <Box className="fn_cs_contact_form" gap={3}>
            <input
              type="text"
              placeholder="Recipient Address"
              className="input"
              onChange={handleChaneAddress}
              disabled={isLoading}
              style={{ fontSize: '14px', color: 'lightgray' }}
            />
            <input
              type="number"
              placeholder="Amount to send (CRNL)"
              className="input"
              onChange={handleChangeAmount}
              disabled={isLoading}
              style={{ fontSize: '14px', color: 'lightgray' }}
            />
          </Box>
        </Box>
        <button
          className="metaportal_fn_button full"
          style={{
            border: 'none',
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}
          disabled={isLoading}
          onClick={handleSendToken}
        >
          <Box
            mx={2}
            display="flex"
            alignItems="center"
            justifyContent="center"
            width={100}
            position="relative"
          >
            <span>Send</span>
            {isLoading && (
              <Box
                display={'flex'}
                position="absolute"
                left="100%"
                top="50%"
                sx={{ transform: 'translate(-50%, -50%)' }}
              >
                <CircularProgress size={24} />
              </Box>
            )}
          </Box>
        </button>
      </Box>
    </Dialog>
  );
};
