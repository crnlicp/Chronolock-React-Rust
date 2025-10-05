import { Principal } from '@dfinity/principal';
import { useActor } from '../ActorContextProvider';
import { useAuth } from './useAuth';
import { useCallback, useEffect, useMemo, useRef } from 'react';
import { useLocation } from 'react-router';

export interface IUseCrnlToken {
  isLoading: boolean;
  balanceData: string;
  balanceRaw: bigint;
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
  deductFromBalanceData: unknown;
  isDeductFromBalanceLoading: boolean;
  deductFromBalanceError: Error | undefined;
  // New statistics functions
  totalSupplyData: unknown;
  isTotalSupplyLoading: boolean;
  totalSupplyError: Error | undefined;
  totalBurnedData: unknown;
  isTotalBurnedLoading: boolean;
  totalBurnedError: Error | undefined;
  claimReferral: () => Promise<unknown>;
  getRefrrealCode: () => Promise<unknown>;
  getFee: () => Promise<unknown>;
  registerUser: () => Promise<unknown>;
  checkBalance: () => Promise<unknown>;
  transfer: (transferArgs: ITransferArgs) => Promise<unknown>;
  // New statistics functions
  getTotalSupply: () => Promise<unknown>;
  getTotalBurned: () => Promise<unknown>;
  deductFromBalance: (amount: bigint, description: string) => Promise<unknown>;
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

  const accountPrincipal = useMemo(
    () => Principal.fromText(principal ?? 'aaaaa-aa'),
    [principal],
  );

  const createAccountPayload = useCallback(() => {
    return {
      owner: accountPrincipal,
      subaccount: [] as [],
    };
  }, [accountPrincipal]);

  const accountArgs = useMemo(() => [createAccountPayload()], [createAccountPayload]);

  const referrerCode = location.search
    ? new URLSearchParams(location.search).get('referral_code')
    : '';

  const parseToBigInt = (value: unknown): bigint => {
    if (typeof value === 'bigint') {
      return value;
    }
    if (typeof value === 'number') {
      if (!Number.isFinite(value)) {
        return 0n;
      }
      return BigInt(Math.trunc(value));
    }
    if (typeof value === 'string') {
      const sanitized = value.replace(/[_,\s,]/g, '');
      if (/^\d+$/.test(sanitized)) {
        return BigInt(sanitized);
      }
      return 0n;
    }
    if (value && typeof value === 'object' && 'toString' in value) {
      const stringValue = (value as { toString: () => string }).toString();
      if (/^\d+$/.test(stringValue)) {
        return BigInt(stringValue);
      }
    }
    return 0n;
  };

  const {
    call: checkBalance,
    loading: isBalanceLoading,
    data: balance,
    error: balanceError,
  } = crnlQueryCall({
    refetchOnMount: false,
    functionName: 'icrc1_balance_of' as any,
    args: accountArgs,
  });

  const balanceRaw = (() => {
    try {
      return balance !== undefined && balance !== null
        ? parseToBigInt(balance)
        : 0n;
    } catch (error) {
      console.warn('Unable to parse balance as bigint', error);
      return 0n;
    }
  })();

  const readableBalance = (rawBalance: bigint) => {
    const numericBalance = Number(rawBalance) / 1e8;
    if (!Number.isFinite(numericBalance)) {
      return rawBalance.toString();
    }
    return numericBalance.toLocaleString(undefined, {
      minimumFractionDigits: 2,
      maximumFractionDigits: 8,
    });
  };

  const balanceData = readableBalance(balanceRaw);

  const {
    call: getRefrrealCode,
    loading: isReferralLoading,
    data: referralCode,
    error: referralError,
  } = crnlQueryCall({
    refetchOnMount: false,
    functionName: 'get_referral_code' as any,
    args: accountArgs,
  });

  const {
    call: getFee,
    loading: isFeeLoading,
    data: feeData,
    error: feeError,
  } = crnlQueryCall({
    refetchOnMount: false,
    functionName: 'icrc1_fee' as any,
  });

  const {
    call: getTotalSupply,
    loading: isTotalSupplyLoading,
    data: totalSupplyData,
    error: totalSupplyError,
  } = crnlQueryCall({
    refetchOnMount: false,
    functionName: 'icrc1_total_supply' as any,
  });

  const {
    call: getTotalBurned,
    loading: isTotalBurnedLoading,
    data: totalBurnedData,
    error: totalBurnedError,
  } = crnlQueryCall({
    refetchOnMount: false,
    functionName: 'get_total_burned' as any,
  });

  const {
    call: deductFromBalanceCall,
    loading: isDeductFromBalanceLoading,
    data: deductFromBalanceData,
    error: deductFromBalanceError,
  } = crnlUpdateCall({
    functionName: 'deduct_from_balance' as any,
  });

  const deductFromBalance = useCallback(
    async (amount: bigint, description: string) => {
      const callerAccount = createAccountPayload();
      return deductFromBalanceCall([
        {
          caller: callerAccount,
          amount,
          description,
        },
      ]).then((res) => {
        const result = res as Record<string, unknown> | undefined;
        if (result && 'Err' in result) {
          const errValue = result.Err;
          const errorKey =
            errValue && typeof errValue === 'object'
              ? Object.keys(errValue as Record<string, unknown>)[0]
              : 'UnknownError';
          throw new Error(`Failed to deduct balance: ${errorKey}`);
        }
        checkBalance();
        return res;
      });
    },
    [deductFromBalanceCall, createAccountPayload, checkBalance],
  );

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
    [transferCall, checkBalance],
  );

  const isLoading =
    isBalanceLoading ||
    isRegisterLoading ||
    isTransferLoading ||
    isFeeLoading ||
    isReferralLoading ||
    isClaimReferralLoading ||
    isTotalSupplyLoading ||
    isTotalBurnedLoading ||
    isDeductFromBalanceLoading;

  const initialTokenStatsFetched = useRef(false);
  const lastPrincipalFetched = useRef<string | null>(null);
  const lastRegisteredPrincipal = useRef<string | null>(null);
  const lastReferralClaimKey = useRef<string | null>(null);

  useEffect(() => {
    if (initialTokenStatsFetched.current) {
      return;
    }
    initialTokenStatsFetched.current = true;
    getFee();
    getTotalSupply();
    getTotalBurned();
  }, [getFee, getTotalSupply, getTotalBurned]);

  useEffect(() => {
    const currentPrincipal = principal ?? 'anonymous';
    if (lastPrincipalFetched.current === currentPrincipal) {
      return;
    }
    lastPrincipalFetched.current = currentPrincipal;
    checkBalance();
    getRefrrealCode();
  }, [principal, checkBalance, getRefrrealCode]);

  useEffect(() => {
    if (!principal) {
      lastRegisteredPrincipal.current = null;
      if (!referrerCode) {
        lastReferralClaimKey.current = null;
      }
      return;
    }

    if (lastRegisteredPrincipal.current !== principal) {
      lastRegisteredPrincipal.current = principal;
      registerUser([createAccountPayload(), []])
        .then((res) => {
          getRefrrealCode();
          checkBalance();
          console.log('User registered', res);
        })
        .catch((err) => {
          console.error('Error registering user:', err);
        });
    }

    if (referrerCode) {
      const claimKey = `${principal}::${referrerCode}`;
      if (lastReferralClaimKey.current === claimKey) {
        return;
      }
      lastReferralClaimKey.current = claimKey;
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
    } else {
      lastReferralClaimKey.current = null;
    }
  }, [principal, referrerCode, registerUser, claimReferral, createAccountPayload, getRefrrealCode, checkBalance]);

  return {
    isLoading,
    balanceData,
    balanceRaw,
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
    deductFromBalanceData,
    isDeductFromBalanceLoading,
    deductFromBalanceError,
    totalSupplyData,
    isTotalSupplyLoading,
    totalSupplyError,
    totalBurnedData,
    isTotalBurnedLoading,
    totalBurnedError,
    claimReferral,
    getRefrrealCode,
    getFee,
    registerUser,
    checkBalance,
    transfer,
    getTotalSupply,
    getTotalBurned,
    deductFromBalance,
  };
};
