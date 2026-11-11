//! Chronolock records and their indexes.

use crate::auth::is_admin;
use crate::media;
use crate::state::{
    config, next_lock_id, CREATORS, LOCKS, OWNER_INDEX, PUBLIC_INDEX, RECIPIENT_INDEX,
};
use crate::types::{
    ChronoError, ChronoResult, Chronolock, CreateChronolockArgs, LockPage, MediaRef, Page, Stats,
    PUBLIC_RECIPIENT,
};
use candid::Principal;
use std::collections::BTreeSet;

pub const MAX_PAGE: u64 = 100;

/// A lock request that passed every check that does not need an `await`.
/// Creating it never fails, so a payment taken before `insert` is never lost.
pub struct ValidatedLock {
    pub unlock_time: u64,
    pub user_keys: Vec<crate::types::UserKey>,
    pub encrypted_metadata: String,
    pub media: Option<MediaRef>,
    pub recipients: BTreeSet<Principal>,
    pub is_public: bool,
}

pub fn validate(caller: Principal, args: CreateChronolockArgs) -> ChronoResult<ValidatedLock> {
    let limits = config().limits;
    if args.scheme != crate::vetkd::SCHEME_VERSION {
        return Err(ChronoError::InvalidInput(format!(
            "unsupported encryption scheme {}; please reload the app",
            args.scheme
        )));
    }
    if args.encrypted_metadata.is_empty() {
        return Err(ChronoError::InvalidInput(
            "encrypted metadata is required".to_string(),
        ));
    }
    let now_s = ic_cdk::api::time() / 1_000_000_000;
    if args.unlock_time <= now_s {
        return Err(ChronoError::InvalidInput(
            "unlock time must be in the future".to_string(),
        ));
    }
    if args.user_keys.is_empty() {
        return Err(ChronoError::InvalidInput(
            "at least one key is required".to_string(),
        ));
    }
    if args.user_keys.len() > limits.max_recipients as usize {
        return Err(ChronoError::LimitExceeded(format!(
            "at most {} recipients",
            limits.max_recipients
        )));
    }
    let size = args.encrypted_metadata.len()
        + args
            .user_keys
            .iter()
            .map(|k| k.user.len() + k.key.len())
            .sum::<usize>();
    if size as u64 > limits.max_metadata_bytes {
        return Err(ChronoError::LimitExceeded("metadata too large".to_string()));
    }

    let mut recipients = BTreeSet::new();
    let mut is_public = false;
    for k in &args.user_keys {
        if k.user == PUBLIC_RECIPIENT {
            is_public = true;
        } else {
            let p = Principal::from_text(&k.user)
                .map_err(|_| ChronoError::InvalidInput(format!("invalid recipient {}", k.user)))?;
            if p == Principal::anonymous() {
                return Err(ChronoError::InvalidInput("anonymous recipient".to_string()));
            }
            if !recipients.insert(p) {
                return Err(ChronoError::InvalidInput(format!(
                    "duplicate recipient {}",
                    k.user
                )));
            }
        }
    }

    let media = match &args.media_id {
        Some(id) => Some(media::check_attachable(*id, caller)?),
        None => None,
    };

    Ok(ValidatedLock {
        unlock_time: args.unlock_time,
        user_keys: args.user_keys,
        encrypted_metadata: args.encrypted_metadata,
        media,
        recipients,
        is_public,
    })
}

pub fn insert(
    caller: Principal,
    v: ValidatedLock,
    payment: Option<crate::payments::PaymentReceipt>,
) -> Chronolock {
    let id = next_lock_id();
    if let Some(m) = &v.media {
        media::attach(m.media_id, id);
    }
    let lock = Chronolock {
        id,
        owner: caller,
        creator: caller,
        scheme: crate::vetkd::SCHEME_VERSION,
        unlock_time: v.unlock_time,
        created_at: ic_cdk::api::time() / 1_000_000,
        user_keys: v.user_keys,
        encrypted_metadata: v.encrypted_metadata,
        media: v.media,
        payment,
    };
    LOCKS.with(|l| l.borrow_mut().insert(id, lock.clone()));
    OWNER_INDEX.with(|o| o.borrow_mut().insert((caller, id), ()));
    RECIPIENT_INDEX.with(|r| {
        let mut r = r.borrow_mut();
        for p in &v.recipients {
            r.insert((*p, id), ());
        }
    });
    if v.is_public {
        PUBLIC_INDEX.with(|p| p.borrow_mut().insert(id, ()));
    }
    CREATORS.with(|c| {
        let mut c = c.borrow_mut();
        let n = c.get(&caller).unwrap_or(0);
        c.insert(caller, n + 1);
    });
    ic_cdk::println!("lock {id} created by {caller}");
    lock
}

pub fn get(id: u64) -> Option<Chronolock> {
    LOCKS.with(|l| l.borrow().get(&id))
}

pub fn burn(caller: Principal, id: u64) -> ChronoResult<()> {
    let lock = get(id).ok_or(ChronoError::NotFound)?;
    if lock.owner != caller && !is_admin(&caller) {
        return Err(ChronoError::Unauthorized);
    }
    LOCKS.with(|l| l.borrow_mut().remove(&id));
    OWNER_INDEX.with(|o| o.borrow_mut().remove(&(lock.owner, id)));
    RECIPIENT_INDEX.with(|r| {
        let mut r = r.borrow_mut();
        for k in &lock.user_keys {
            if let Ok(p) = Principal::from_text(&k.user) {
                r.remove(&(p, id));
            }
        }
    });
    PUBLIC_INDEX.with(|p| p.borrow_mut().remove(&id));
    if let Some(m) = &lock.media {
        media::delete(m.media_id, m.chunk_count);
    }
    ic_cdk::println!("lock {id} burned by {caller}");
    Ok(())
}

pub fn transfer_owner(id: u64, from: Principal, to: Principal) -> ChronoResult<()> {
    let mut lock = get(id).ok_or(ChronoError::NotFound)?;
    if lock.owner != from {
        return Err(ChronoError::Unauthorized);
    }
    lock.owner = to;
    LOCKS.with(|l| l.borrow_mut().insert(id, lock));
    OWNER_INDEX.with(|o| {
        let mut o = o.borrow_mut();
        o.remove(&(from, id));
        o.insert((to, id), ());
    });
    Ok(())
}

fn clamp(page: &Page) -> (usize, usize) {
    (page.offset as usize, page.limit.min(MAX_PAGE) as usize)
}

/// All locks, newest first.
pub fn list_all(page: Page) -> LockPage {
    let (skip, take) = clamp(&page);
    LOCKS.with(|l| {
        let l = l.borrow();
        LockPage {
            total: l.len(),
            items: l
                .iter()
                .rev()
                .skip(skip)
                .take(take)
                .map(|e| e.value())
                .collect(),
        }
    })
}

type PrincipalIndex = std::cell::RefCell<
    ic_stable_structures::StableBTreeMap<(Principal, u64), (), crate::state::Memory>,
>;

fn ids_for(
    principal: Principal,
    index: &'static std::thread::LocalKey<PrincipalIndex>,
) -> Vec<u64> {
    index.with(|i| {
        i.borrow()
            .range((principal, 0)..=(principal, u64::MAX))
            .map(|e| e.key().1)
            .collect()
    })
}

fn page_of(mut ids: Vec<u64>, page: Page) -> LockPage {
    ids.sort_unstable_by(|a, b| b.cmp(a));
    let (skip, take) = clamp(&page);
    LockPage {
        total: ids.len() as u64,
        items: ids
            .into_iter()
            .skip(skip)
            .take(take)
            .filter_map(get)
            .collect(),
    }
}

pub fn list_owned(owner: Principal, page: Page) -> LockPage {
    page_of(ids_for(owner, &OWNER_INDEX), page)
}

/// Locks addressed to `user` (private) plus public ones, newest first.
/// With `unlocked_only`, only locks whose unlock time has passed.
pub fn list_for_recipient(user: Principal, unlocked_only: bool, page: Page) -> LockPage {
    let mut ids: BTreeSet<u64> = ids_for(user, &RECIPIENT_INDEX).into_iter().collect();
    PUBLIC_INDEX.with(|p| ids.extend(p.borrow().iter().map(|e| *e.key())));
    let now_s = ic_cdk::api::time() / 1_000_000_000;
    let ids: Vec<u64> = ids
        .into_iter()
        .filter(|id| !unlocked_only || get(*id).is_some_and(|l| l.unlock_time <= now_s))
        .collect();
    page_of(ids, page)
}

pub fn count_owned(owner: Principal) -> u64 {
    ids_for(owner, &OWNER_INDEX).len() as u64
}

pub fn stats() -> Stats {
    let media = crate::state::MEDIA_SESSIONS.with(|s| {
        s.borrow()
            .iter()
            .filter(|e| e.value().lock_id.is_some())
            .count() as u64
    });
    Stats {
        total_locks: LOCKS.with(|l| l.borrow().len()),
        unique_creators: CREATORS.with(|c| c.borrow().len()),
        media_locks: media,
        public_locks: PUBLIC_INDEX.with(|p| p.borrow().len()),
    }
}
