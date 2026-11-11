//! Stable-memory layout. Every map lives in its own virtual memory, so the
//! canister survives upgrades without any pre/post-upgrade serialization.
//!
//! Memory ids are part of the on-chain layout: never reuse or renumber them.

use crate::types::{Chronolock, Config, MediaSession};
use candid::Principal;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell};
use std::cell::RefCell;

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

const MEM_CONFIG: MemoryId = MemoryId::new(0);
const MEM_ADMINS: MemoryId = MemoryId::new(1);
const MEM_LOCKS: MemoryId = MemoryId::new(2);
const MEM_OWNER_INDEX: MemoryId = MemoryId::new(3);
const MEM_RECIPIENT_INDEX: MemoryId = MemoryId::new(4);
const MEM_PUBLIC_INDEX: MemoryId = MemoryId::new(5);
const MEM_CREATORS: MemoryId = MemoryId::new(6);
const MEM_MEDIA_SESSIONS: MemoryId = MemoryId::new(7);
const MEM_MEDIA_CHUNKS: MemoryId = MemoryId::new(8);
// 9.. reserved for payments (see payments.rs).

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    pub static CONFIG: RefCell<StableCell<Config, Memory>> = RefCell::new(
        StableCell::init(memory(MEM_CONFIG), Config::default())
    );

    /// Extra admins (controllers are always admins).
    pub static ADMINS: RefCell<StableBTreeMap<Principal, (), Memory>> =
        RefCell::new(StableBTreeMap::init(memory(MEM_ADMINS)));

    pub static LOCKS: RefCell<StableBTreeMap<u64, Chronolock, Memory>> =
        RefCell::new(StableBTreeMap::init(memory(MEM_LOCKS)));

    /// (owner, lock id)
    pub static OWNER_INDEX: RefCell<StableBTreeMap<(Principal, u64), (), Memory>> =
        RefCell::new(StableBTreeMap::init(memory(MEM_OWNER_INDEX)));

    /// (recipient, lock id) for private recipients.
    pub static RECIPIENT_INDEX: RefCell<StableBTreeMap<(Principal, u64), (), Memory>> =
        RefCell::new(StableBTreeMap::init(memory(MEM_RECIPIENT_INDEX)));

    /// Locks anyone can open after the unlock time.
    pub static PUBLIC_INDEX: RefCell<StableBTreeMap<u64, (), Memory>> =
        RefCell::new(StableBTreeMap::init(memory(MEM_PUBLIC_INDEX)));

    /// creator -> number of locks created (for stats).
    pub static CREATORS: RefCell<StableBTreeMap<Principal, u64, Memory>> =
        RefCell::new(StableBTreeMap::init(memory(MEM_CREATORS)));

    pub static MEDIA_SESSIONS: RefCell<StableBTreeMap<u64, MediaSession, Memory>> =
        RefCell::new(StableBTreeMap::init(memory(MEM_MEDIA_SESSIONS)));

    /// (media id, chunk index) -> bytes
    pub static MEDIA_CHUNKS: RefCell<StableBTreeMap<(u64, u32), Vec<u8>, Memory>> =
        RefCell::new(StableBTreeMap::init(memory(MEM_MEDIA_CHUNKS)));
}

pub fn memory(id: MemoryId) -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(id))
}

pub fn config() -> Config {
    CONFIG.with(|c| c.borrow().get().clone())
}

pub fn update_config(f: impl FnOnce(&mut Config)) {
    CONFIG.with(|c| {
        let mut cell = c.borrow_mut();
        let mut cfg = cell.get().clone();
        f(&mut cfg);
        cell.set(cfg);
    });
}

pub fn next_lock_id() -> u64 {
    let mut id = 0;
    update_config(|c| {
        id = c.next_lock_id;
        c.next_lock_id += 1;
    });
    id
}

pub fn next_media_id() -> u64 {
    let mut id = 0;
    update_config(|c| {
        id = c.next_media_id;
        c.next_media_id += 1;
    });
    id
}

pub fn next_tx_id() -> u64 {
    let mut id = 0;
    update_config(|c| {
        id = c.next_tx_id;
        c.next_tx_id += 1;
    });
    id
}
