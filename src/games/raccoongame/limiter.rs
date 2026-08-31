use std::num::NonZeroU32;
use std::sync::LazyLock;
use governor::{Quota, RateLimiter};

pub const RETRY_BACKOFF_ENABLED: bool = true;

pub static MAIL_TM_LIMITER: LazyLock<RateLimiter<governor::state::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>> =
    LazyLock::new(|| {
        let quota = Quota::per_second(NonZeroU32::try_from(6u32).unwrap())
            .allow_burst(NonZeroU32::try_from(1u32).unwrap());

        RateLimiter::direct(quota)
    });

pub static MAIL_GW_LIMITER: LazyLock<RateLimiter<governor::state::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>> =
    LazyLock::new(|| {
        let quota = Quota::per_second(NonZeroU32::try_from(6u32).unwrap())
            .allow_burst(NonZeroU32::try_from(1u32).unwrap());

        RateLimiter::direct(quota)
    });

pub static RACCOON_GAME_LIMITER: LazyLock<RateLimiter<governor::state::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>> =
    LazyLock::new(|| {
        let quota = Quota::per_second(NonZeroU32::try_from(1u32).unwrap())
            .allow_burst(NonZeroU32::try_from(1u32).unwrap());

        RateLimiter::direct(quota)
    });
