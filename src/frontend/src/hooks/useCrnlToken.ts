import { Principal } from '@dfinity/principal';
import { useActor } from '../ActorContextProvider';
import { useAuth } from './useAuth';
import { useEffect } from 'react';

export interface IUseCrnlToken {
  balanceData: string;
  isLoading: boolean;
  balanceError: Error | undefined;
  registerData: unknown;
  isRegisterLoading: boolean;
  registerError: Error | undefined;
  registerUser: () => Promise<unknown>;
  checkBalance: () => Promise<unknown>;
}

export const useCrnlToken = (): IUseCrnlToken => {
  const {
    crnlLedgerActor: { useQueryCall: crnlQueryCall },
  } = useActor();

  const { principal } = useAuth();

  const {
    call: checkBalance,
    data: balance,
    error: balanceError,
    loading: isBalanceLoading,
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
    call: registerUser,
    data: registerData,
    error: registerError,
    loading: isRegisterLoading,
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

  const isLoading = isBalanceLoading;

  useEffect(() => {
    if (principal) {
      registerUser().then(() => {
        checkBalance();
        console.log('User registered successfully');
      });
    }
  }, [principal]);

  return {
    balanceData,
    isLoading,
    balanceError,
    registerData,
    isRegisterLoading,
    registerError,
    registerUser,
    checkBalance,
  };
};
