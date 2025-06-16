import { Principal } from '@dfinity/principal';
import { useActor } from '../ActorContextProvider';
import { useAuth } from './useAuth';
import { useCallback, useEffect } from 'react';
import { useLocation } from 'react-router';

export interface IUseCrnlToken {
  isLoading: boolean;
  balanceData: string;
  isBalanceLoading: boolean;
  balanceError: Error | undefined;
  registerData: unknown;
  isRegisterLoading: boolean;
  registerError: Error | undefined;
  transferData: unknown;
  isTransferLoading: boolean;
  transferError: Error | undefined;
  feeData: unknown;
  isFeeLoading: boolean;
  feeError: Error | undefined;
  referralCode: unknown;
  isReferralLoading: boolean;
  referralError: Error | undefined;
  claimReferralData: unknown;
  isClaimReferralLoading: boolean;
  claimReferralError: Error | undefined;
  claimReferral: () => Promise<unknown>;
  getRefrrealCode: () => Promise<unknown>;
  getFee: () => Promise<unknown>;
  registerUser: () => Promise<unknown>;
  checkBalance: () => Promise<unknown>;
  transfer: (transferArgs: ITransferArgs) => Promise<unknown>;
}

interface ITransferArgs {
  to: Principal;
  amount: BigInt;
}

export const useCrnlToken = (): IUseCrnlToken => {
  const {
    crnlLedgerActor: {
      useQueryCall: crnlQueryCall,
      useUpdateCall: crnlUpdateCall,
    },
  } = useActor();

  const location = useLocation();
  const { principal } = useAuth();

  const referrerCode = location.search
    ? new URLSearchParams(location.search).get('referral_code')
    : '';

  const {
    call: checkBalance,
    loading: isBalanceLoading,
    data: balance,
    error: balanceError,
  } = crnlQueryCall({
    refetchOnMount: true,
    functionName: 'icrc1_balance_of' as any,
    args: [
      {
        owner: Principal.fromText(principal ?? 'aaaaa-aa'),
        subaccount: [],
      },
    ],
  });

  const {
    call: getRefrrealCode,
    loading: isReferralLoading,
    data: referralCode,
    error: referralError,
  } = crnlQueryCall({
    refetchOnMount: true,
    functionName: 'get_referral_code' as any,
    args: [
      {
        owner: Principal.fromText(principal ?? 'aaaaa-aa'),
        subaccount: [],
      },
    ],
  });

  const {
    call: getFee,
    loading: isFeeLoading,
    data: feeData,
    error: feeError,
  } = crnlQueryCall({
    refetchOnMount: true,
    functionName: 'icrc1_fee' as any,
  });

  const {
    call: transferCall,
    loading: isTransferLoading,
    data: transferData,
    error: transferError,
  } = crnlUpdateCall({
    functionName: 'icrc1_transfer' as any,
  });

  const {
    call: registerUser,
    data: registerData,
    loading: isRegisterLoading,
    error: registerError,
  } = crnlUpdateCall({
    functionName: 'register_user' as any,
  });

  const {
    call: claimReferral,
    data: claimReferralData,
    loading: isClaimReferralLoading,
    error: claimReferralError,
  } = crnlUpdateCall({
    functionName: 'claim_referral' as any,
  });

  const transfer = useCallback(
    async (args: ITransferArgs) => {
      return transferCall([
        {
          to: {
            owner: Principal.fromText(args.to.toText()),
            subaccount: [],
          },
          from_subaccount: [],
          amount: args.amount,
        },
        [],
      ]).then((res) => {
        checkBalance();
        return res;
      });
    },
    [transferCall],
  );

  const readableBalance = (balance: unknown) => {
    return (Number(balance) / 1e8).toLocaleString(undefined, {
      minimumFractionDigits: 2,
      maximumFractionDigits: 8,
    });
  };

  const balanceData = balance ? readableBalance(balance) : '0.00';

  const isLoading =
    isBalanceLoading ||
    isRegisterLoading ||
    isTransferLoading ||
    isFeeLoading ||
    isReferralLoading ||
    isClaimReferralLoading;

  useEffect(() => {
    if (principal) {
      registerUser([
        {
          owner: Principal.fromText(principal ?? 'aaaaa-aa'),
          subaccount: [],
        },
        [],
      ])
        .then((res) => {
          getRefrrealCode();
          checkBalance();
          console.log('User registered', res);
        })
        .catch((err) => {
          console.error('Error registering user:', err);
        });
    }
    if (principal && referrerCode) {
      claimReferral([
        {
          referral_code: referrerCode,
        },
      ])
        .then((res) => {
          console.log('Referral claimed', res);
        })
        .catch((err) => {
          console.error('Error claiming referral:', err);
        });
    }
  }, [principal]);

  console.log(balance, balanceData, 'balanceData');
  console.log(transferData, 'transferData');
  console.log(feeData, 'feeData');
  console.log(referralCode, 'referralCode');
  console.log(location.pathname, 'location.pathname');
  console.log(location.search, 'location.search');
  console.log(referrerCode, 'referrerCode');
  console.log(claimReferralData, 'claimReferralData');

  return {
    isLoading,
    balanceData,
    isBalanceLoading,
    balanceError,
    registerData,
    isRegisterLoading,
    registerError,
    transferData,
    isTransferLoading,
    transferError,
    feeData,
    isFeeLoading,
    feeError,
    referralCode,
    isReferralLoading,
    referralError,
    claimReferralData,
    isClaimReferralLoading,
    claimReferralError,
    claimReferral,
    getRefrrealCode,
    getFee,
    registerUser,
    checkBalance,
    transfer,
  };
};
