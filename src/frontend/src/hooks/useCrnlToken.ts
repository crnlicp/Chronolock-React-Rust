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
  registerUser: () => Promise<unknown>;
  checkBalance: () => Promise<unknown>;
  transfer: (transferArgs: ITransferArgs) => Promise<void>;
}

// 5lvul-lqobm-ks5e6-dqehb-ckzns-j6utt-objsh-oht4p-uea4x-lbrtp-gqe
// dfi5r-jv4p3-4ipkk-3py6c-dqgfe-uerxt-stprr-tzq35-tnhig-yetrp-yqe
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
      setTimeout(() => {
        transferCall().then(() => {
          checkBalance();
        });
      }, 0);
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

  const isLoading = isBalanceLoading || isRegisterLoading || isTransferLoading;

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

  console.log(balance);

  return {
    balanceData,
    isLoading,
    balanceError,
    registerData,
    registerError,
    transferData,
    transferError,
    registerUser,
    checkBalance,
    transfer,
  };
};
