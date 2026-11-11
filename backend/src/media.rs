//! Chunked upload of (already encrypted) media.
//!
//! Flow: `start_media_upload` -> `upload_media_chunk` x N -> `finish_media_upload`
//! -> reference the media id in `create_chronolock`. Sessions that are never
//! attached to a chronolock expire and are deleted by a timer.

use crate::auth::{authenticated_caller, is_in_flight};
use crate::state::{config, next_media_id, MEDIA_CHUNKS, MEDIA_SESSIONS};
use crate::types::{
    ChronoError, ChronoResult, MediaInfo, MediaRef, MediaSession, StartMediaUploadArgs,
};
use candid::Principal;

fn open_sessions_of(owner: Principal) -> u32 {
    MEDIA_SESSIONS.with(|s| {
        s.borrow()
            .iter()
            .filter(|e| {
                let v = e.value();
                v.owner == owner && v.lock_id.is_none()
            })
            .count() as u32
    })
}

pub fn start(args: StartMediaUploadArgs) -> ChronoResult<u64> {
    let caller = authenticated_caller()?;
    let limits = config().limits;
    if args.size == 0 || args.size > limits.max_media_bytes {
        return Err(ChronoError::LimitExceeded(format!(
            "media must be between 1 and {} bytes",
            limits.max_media_bytes
        )));
    }
    let min_chunks = args.size.div_ceil(limits.max_chunk_bytes);
    if args.chunk_count == 0
        || (args.chunk_count as u64) < min_chunks
        || (args.chunk_count as u64) > args.size
    {
        return Err(ChronoError::InvalidInput("invalid chunk_count".to_string()));
    }
    if open_sessions_of(caller) >= limits.max_open_uploads_per_user {
        return Err(ChronoError::LimitExceeded(
            "too many unfinished uploads; finish or cancel one first".to_string(),
        ));
    }
    let id = next_media_id();
    MEDIA_SESSIONS.with(|s| {
        s.borrow_mut().insert(
            id,
            MediaSession {
                owner: caller,
                size: args.size,
                chunk_count: args.chunk_count,
                received: vec![false; args.chunk_count as usize],
                created_at_ns: ic_cdk::api::time(),
                finished: false,
                lock_id: None,
            },
        )
    });
    Ok(id)
}

fn owned_open_session(media_id: u64, caller: Principal) -> ChronoResult<MediaSession> {
    let session = MEDIA_SESSIONS
        .with(|s| s.borrow().get(&media_id))
        .ok_or(ChronoError::NotFound)?;
    if session.owner != caller {
        return Err(ChronoError::Unauthorized);
    }
    if session.lock_id.is_some() {
        return Err(ChronoError::InvalidInput(
            "media is already attached".to_string(),
        ));
    }
    Ok(session)
}

/// Media of a caller with a payment in flight must not change under it.
fn not_in_flight(caller: Principal) -> ChronoResult<()> {
    if is_in_flight(&caller) {
        return Err(ChronoError::AlreadyProcessing);
    }
    Ok(())
}

pub fn upload_chunk(media_id: u64, index: u32, chunk: Vec<u8>) -> ChronoResult<()> {
    let caller = authenticated_caller()?;
    not_in_flight(caller)?;
    let mut session = owned_open_session(media_id, caller)?;
    if session.finished {
        return Err(ChronoError::InvalidInput(
            "upload already finished".to_string(),
        ));
    }
    if index >= session.chunk_count {
        return Err(ChronoError::InvalidInput(
            "chunk index out of range".to_string(),
        ));
    }
    if chunk.is_empty() || chunk.len() as u64 > config().limits.max_chunk_bytes {
        return Err(ChronoError::LimitExceeded(
            "chunk too large or empty".to_string(),
        ));
    }
    // Overwriting a chunk is allowed (retries); total size is checked on finish.
    MEDIA_CHUNKS.with(|c| c.borrow_mut().insert((media_id, index), chunk));
    session.received[index as usize] = true;
    MEDIA_SESSIONS.with(|s| s.borrow_mut().insert(media_id, session));
    Ok(())
}

fn stored_size(media_id: u64, chunk_count: u32) -> u64 {
    MEDIA_CHUNKS.with(|c| {
        let c = c.borrow();
        (0..chunk_count)
            .filter_map(|i| c.get(&(media_id, i)))
            .map(|v| v.len() as u64)
            .sum()
    })
}

pub fn finish(media_id: u64) -> ChronoResult<MediaInfo> {
    let caller = authenticated_caller()?;
    not_in_flight(caller)?;
    let mut session = owned_open_session(media_id, caller)?;
    if session.received.iter().any(|r| !r) {
        return Err(ChronoError::InvalidInput(
            "not all chunks were uploaded".to_string(),
        ));
    }
    if stored_size(media_id, session.chunk_count) != session.size {
        return Err(ChronoError::InvalidInput(
            "uploaded size does not match".to_string(),
        ));
    }
    session.finished = true;
    MEDIA_SESSIONS.with(|s| s.borrow_mut().insert(media_id, session.clone()));
    Ok(to_info(media_id, &session))
}

pub fn cancel(media_id: u64) -> ChronoResult<()> {
    let caller = authenticated_caller()?;
    not_in_flight(caller)?;
    let session = owned_open_session(media_id, caller)?;
    delete(media_id, session.chunk_count);
    Ok(())
}

/// Validates that `media_id` can be attached to a new lock by `caller`.
pub fn check_attachable(media_id: u64, caller: Principal) -> ChronoResult<MediaRef> {
    let session = owned_open_session(media_id, caller)?;
    if !session.finished {
        return Err(ChronoError::InvalidInput(
            "media upload is not finished".to_string(),
        ));
    }
    Ok(MediaRef {
        media_id,
        size: session.size,
        chunk_count: session.chunk_count,
    })
}

pub fn attach(media_id: u64, lock_id: u64) {
    MEDIA_SESSIONS.with(|s| {
        let mut s = s.borrow_mut();
        if let Some(mut session) = s.get(&media_id) {
            session.lock_id = Some(lock_id);
            s.insert(media_id, session);
        }
    });
}

pub fn delete(media_id: u64, chunk_count: u32) {
    MEDIA_CHUNKS.with(|c| {
        let mut c = c.borrow_mut();
        for i in 0..chunk_count {
            c.remove(&(media_id, i));
        }
    });
    MEDIA_SESSIONS.with(|s| s.borrow_mut().remove(&media_id));
}

/// Media is encrypted client-side, so attached media is readable by anyone;
/// unattached uploads are only visible to their owner.
pub fn get_chunk(media_id: u64, index: u32) -> ChronoResult<Vec<u8>> {
    let session = MEDIA_SESSIONS
        .with(|s| s.borrow().get(&media_id))
        .ok_or(ChronoError::NotFound)?;
    if session.lock_id.is_none() && session.owner != ic_cdk::api::msg_caller() {
        return Err(ChronoError::Unauthorized);
    }
    MEDIA_CHUNKS
        .with(|c| c.borrow().get(&(media_id, index)))
        .ok_or(ChronoError::NotFound)
}

pub fn info(media_id: u64) -> ChronoResult<MediaInfo> {
    let session = MEDIA_SESSIONS
        .with(|s| s.borrow().get(&media_id))
        .ok_or(ChronoError::NotFound)?;
    Ok(to_info(media_id, &session))
}

fn to_info(media_id: u64, s: &MediaSession) -> MediaInfo {
    MediaInfo {
        media_id,
        owner: s.owner,
        size: s.size,
        chunk_count: s.chunk_count,
        finished: s.finished,
        lock_id: s.lock_id,
    }
}

/// Deletes unattached sessions older than the configured TTL.
pub fn cleanup_expired() {
    let ttl_ns = config().limits.upload_ttl_seconds * 1_000_000_000;
    let now = ic_cdk::api::time();
    let expired: Vec<(u64, u32)> = MEDIA_SESSIONS.with(|s| {
        s.borrow()
            .iter()
            .filter(|e| {
                let v = e.value();
                v.lock_id.is_none()
                    && now.saturating_sub(v.created_at_ns) > ttl_ns
                    && !is_in_flight(&v.owner)
            })
            .map(|e| (*e.key(), e.value().chunk_count))
            .collect()
    });
    for (id, n) in expired {
        delete(id, n);
    }
}
