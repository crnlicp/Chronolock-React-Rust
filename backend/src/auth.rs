//! Access control helpers (see the canister-security skill).

use crate::state::ADMINS;
use crate::types::{ChronoError, ChronoResult};
use candid::Principal;
use ic_cdk::api::{is_controller, msg_caller};
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};

/// Returns the caller, rejecting the anonymous principal.
pub fn authenticated_caller() -> ChronoResult<Principal> {
    let caller = msg_caller();
    if caller == Principal::anonymous() {
        return Err(ChronoError::AnonymousCaller);
    }
    Ok(caller)
}

pub fn is_admin(p: &Principal) -> bool {
    *p != Principal::anonymous()
        && (is_controller(p) || ADMINS.with(|a| a.borrow().contains_key(p)))
}

pub fn admin_caller() -> ChronoResult<Principal> {
    let caller = authenticated_caller()?;
    if !is_admin(&caller) {
        return Err(ChronoError::Unauthorized);
    }
    Ok(caller)
}

thread_local! {
    static IN_FLIGHT: RefCell<BTreeSet<Principal>> = const { RefCell::new(BTreeSet::new()) };
    /// caller -> (window start in seconds, count). Heap only: resets on upgrade.
    static RATE: RefCell<HashMap<Principal, (u64, u32)>> = RefCell::new(HashMap::new());
}

/// Allows one in-flight async operation per caller. Released on drop, which
/// also runs if the callback after an `await` traps.
pub struct CallerGuard {
    principal: Principal,
}

impl CallerGuard {
    pub fn new(principal: Principal) -> ChronoResult<Self> {
        IN_FLIGHT.with(|set| {
            if !set.borrow_mut().insert(principal) {
                return Err(ChronoError::AlreadyProcessing);
            }
            Ok(Self { principal })
        })
    }
}

impl Drop for CallerGuard {
    fn drop(&mut self) {
        IN_FLIGHT.with(|set| {
            set.borrow_mut().remove(&self.principal);
        });
    }
}

/// True while `p` has an async operation (e.g. a payment) in flight.
pub fn is_in_flight(p: &Principal) -> bool {
    IN_FLIGHT.with(|set| set.borrow().contains(p))
}

/// Simple fixed-window rate limit per caller.
pub fn check_rate_limit(caller: Principal, per_hour: u32) -> ChronoResult<()> {
    let now_s = ic_cdk::api::time() / 1_000_000_000;
    RATE.with(|r| {
        let mut r = r.borrow_mut();
        let entry = r.entry(caller).or_insert((now_s, 0));
        if now_s.saturating_sub(entry.0) >= 3600 {
            *entry = (now_s, 0);
        }
        if entry.1 >= per_hour {
            return Err(ChronoError::LimitExceeded(
                "Too many key requests, try again later".to_string(),
            ));
        }
        entry.1 += 1;
        Ok(())
    })
}
