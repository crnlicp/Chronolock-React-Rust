import { Principal } from '@dfinity/principal';
import { useActor } from '../ActorContextProvider';
import { useAuth } from './useAuth';
import { useCallback, useEffect, useRef } from 'react';

export interface IUseCrnlToken {
  balanceData: string;
  isLoading: boolean;
  balanceError: Error | undefined;
  registerData: unknown;
  registerError: Error | undefined;
  transferData: unknown;
  transferError: Error | undefined;
  feeData: unknown;
  isFeeLoading: boolean;
  feeError: Error | undefined;
  getFee: () => Promise<unknown>;
  registerUser: () => Promise<unknown>;
  checkBalance: () => Promise<unknown>;
  transfer: (transferArgs: ITransferArgs) => Promise<unknown>;
}

// 10000 3nv5e-pwjqz-56cg7-d6vwf-zf2tf-ofyi3-fgixi-gpykj-somfb-dmyxf-aqe
// 10001 u55pe-inrvr-hzjcl-gxcds-rbhty-ispld-d2o5p-aqp6a-k46uh-xqkvp-zqe
// 10002 mrg7m-blouy-ykmv6-xbmhw-avt2f-g7kqq-hfmoq-gu6nu-gltad-yn5l7-wqe

export interface ITransferArgs {
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

  const transferArgsRef = useRef<ITransferArgs | null>(null);
  const { principal } = useAuth();

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
    call: getFee,
    loading: isFeeLoading,
    data: feeData,
    error: feeError,
  } = crnlQueryCall({
    refetchOnMount: true,
    functionName: 'icrc1_fee' as any,
    args: [],
  });

  const {
    call: transferCall,
    loading: isTransferLoading,
    data: transferData,
    error: transferError,
  } = crnlUpdateCall({
    functionName: 'icrc1_transfer' as any,
    args: transferArgsRef.current
      ? [
          {
            to: {
              owner: Principal.fromText(transferArgsRef.current.to.toText()),
              subaccount: [],
            },
            from_subaccount: [],
            amount: transferArgsRef.current.amount,
          },
          [],
        ]
      : undefined,
  });

  const transfer = useCallback(
    async (args: ITransferArgs) => {
      transferArgsRef.current = args;
      return transferCall().then((res) => {
        checkBalance();
        return res;
      });
    },
    [transferCall],
  );

  const {
    call: registerUser,
    data: registerData,
    loading: isRegisterLoading,
    error: registerError,
  } = crnlQueryCall({
    refetchOnMount: true,
    functionName: 'register_user' as any,
    args: [
      {
        owner: Principal.fromText(principal ?? 'aaaaa-aa'),
        subaccount: [],
      },
      [],
    ],
  });

  const readableBalance = (balance: unknown) => {
    return (Number(balance) / 1e8).toLocaleString(undefined, {
      minimumFractionDigits: 2,
      maximumFractionDigits: 8,
    });
  };

  const balanceData = balance ? readableBalance(balance) : '0.00';

  const isLoading =
    isBalanceLoading || isRegisterLoading || isTransferLoading || isFeeLoading;

  useEffect(() => {
    if (principal) {
      registerUser()
        .then((res) => {
          checkBalance();
          console.log('User registered', res);
        })
        .catch((err) => {
          console.error('Error registering user:', err);
        });
    }
  }, [principal]);

  console.log(balance, balanceData, 'balanceData');
  console.log(transferData, 'transferData');
  console.log(isTransferLoading, 'isTransferLoading');
  console.log(feeData, 'feeData');

  return {
    balanceData,
    isLoading,
    balanceError,
    registerData,
    registerError,
    transferData,
    transferError,
    feeData,
    isFeeLoading,
    feeError,
    getFee,
    registerUser,
    checkBalance,
    transfer,
  };
};
