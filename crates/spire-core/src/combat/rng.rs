//! Immutable project-owned HMAC counter streams (RNG-001), not original-game RNG.
use hmac::{Hmac, Mac};
use sha2::Sha256;

pub const RNG_CONTRACT_ID: &str = "spire-lab-rng-hmac-sha256-v1";
const END: u128 = 1_u128 << 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Domain {
    CombatShuffle,
    CombatEnemyHp,
    CombatEnemyAi,
}

impl Domain {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CombatShuffle => "combat.shuffle",
            Self::CombatEnemyHp => "combat.enemy.hp",
            Self::CombatEnemyAi => "combat.enemy.ai",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Partition {
    Train,
    Selection,
    Final,
    Test,
}

impl Partition {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Train => "train",
            Self::Selection => "selection",
            Self::Final => "final",
            Self::Test => "test",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StreamId {
    domain: Domain,
    scope: Vec<String>,
}

impl StreamId {
    pub fn new(domain: Domain, scope: Vec<String>) -> Result<Self, RngError> {
        let arity = match domain {
            Domain::CombatShuffle => 1,
            Domain::CombatEnemyHp | Domain::CombatEnemyAi => 2,
        };
        if scope.len() != arity {
            return Err(RngError::Invalid("incorrect domain scope arity".into()));
        }
        for field in &scope {
            validate_field(field)?;
        }
        Ok(Self { domain, scope })
    }

    pub fn domain(&self) -> Domain {
        self.domain
    }

    pub fn scope(&self) -> &[String] {
        &self.scope
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StreamState {
    id: StreamId,
    key: [u8; 32],
    word_index: u128,
}

impl StreamState {
    pub fn new(id: StreamId, key: [u8; 32], word_index: u128) -> Result<Self, RngError> {
        if word_index > END {
            return Err(RngError::Invalid("cursor exceeds exhausted sentinel".into()));
        }
        Ok(Self {
            id,
            key,
            word_index,
        })
    }

    pub fn id(&self) -> &StreamId {
        &self.id
    }

    pub fn key(&self) -> &[u8; 32] {
        &self.key
    }

    pub fn word_index(&self) -> u128 {
        self.word_index
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RngError {
    Invalid(String),
    Exhausted { state: StreamState },
}

fn validate_field(field: &str) -> Result<(), RngError> {
    if field.is_empty()
        || field.len() > 255
        || !field.bytes().all(|b| (0x21..=0x7e).contains(&b))
    {
        return Err(RngError::Invalid(
            "identity must be 1–255 printable non-space ASCII bytes".into(),
        ));
    }
    Ok(())
}

pub(super) fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    // HMAC accepts keys of every length; this constructor cannot reject one.
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(message);
    mac.finalize().into_bytes().into()
}

pub fn make_stream(
    root: &[u8; 32],
    partition: Partition,
    episode: &str,
    id: &StreamId,
) -> Result<StreamState, RngError> {
    validate_field(episode)?;
    let mut fields = vec![partition.as_str(), episode, id.domain.as_str()];
    fields.extend(id.scope.iter().map(String::as_str));
    let mut message = b"spire-lab/rng/v1/stream\0".to_vec();
    message.extend((fields.len() as u32).to_be_bytes());
    for field in fields {
        message.extend((field.len() as u32).to_be_bytes());
        message.extend(field.as_bytes());
    }
    StreamState::new(id.clone(), hmac_sha256(root, &message), 0)
}

pub fn draw_u64(state: &StreamState) -> Result<(u64, StreamState), RngError> {
    if state.word_index == END {
        return Err(RngError::Exhausted { state: state.clone() });
    }
    // Validated cursor is below 2**64, so both conversions and increment fit.
    let block = (state.word_index / 4) as u64;
    let lane = (state.word_index % 4) as usize;
    let mut message = b"spire-lab/rng/v1/block\0".to_vec();
    message.extend(block.to_be_bytes());
    let digest = hmac_sha256(&state.key, &message);
    let word = u64::from_be_bytes(digest[lane * 8..lane * 8 + 8].try_into().unwrap());
    let mut next = state.clone();
    next.word_index += 1;
    Ok((word, next))
}

pub(super) fn uniform_from_words(
    bound: u128,
    next: &mut impl FnMut() -> Result<u64, RngError>,
) -> Result<u64, RngError> {
    if !(1..=END).contains(&bound) {
        return Err(RngError::Invalid("bound must be in 1..=2**64".into()));
    }
    if bound == 1 {
        return Ok(0);
    }
    let limit = END - END % bound;
    loop {
        let word = u128::from(next()?);
        if word < limit {
            return Ok((word % bound) as u64);
        }
    }
}

pub fn uniform_below(
    state: &StreamState,
    bound: u128,
) -> Result<(u64, StreamState), RngError> {
    let mut cursor = state.clone();
    let value = uniform_from_words(bound, &mut || {
        let (word, next) = draw_u64(&cursor)?;
        cursor = next;
        Ok(word)
    })?;
    Ok((value, cursor))
}

pub fn bernoulli(
    state: &StreamState,
    numerator: u128,
    denominator: u128,
) -> Result<(bool, StreamState), RngError> {
    if denominator == 0 || denominator > END || numerator > denominator {
        return Err(RngError::Invalid("invalid rational probability".into()));
    }
    if numerator == 0 {
        return Ok((false, state.clone()));
    }
    if numerator == denominator {
        return Ok((true, state.clone()));
    }
    let (ticket, next) = uniform_below(state, denominator)?;
    Ok((u128::from(ticket) < numerator, next))
}

fn weight_sum(weights: &[u128]) -> Result<u128, RngError> {
    let mut total = 0_u128;
    for weight in weights {
        total = total
            .checked_add(*weight)
            .filter(|sum| *sum <= END)
            .ok_or_else(|| RngError::Invalid("weight sum exceeds 2**64".into()))?;
    }
    if total == 0 {
        return Err(RngError::Invalid("weights require a positive sum".into()));
    }
    Ok(total)
}

pub(super) fn weighted_from_ticket(
    weights: &[u128],
    ticket: u64,
) -> Result<usize, RngError> {
    let total = weight_sum(weights)?;
    if u128::from(ticket) >= total {
        return Err(RngError::Invalid("ticket outside weight sum".into()));
    }
    let mut cumulative = 0_u128;
    for (index, weight) in weights.iter().enumerate() {
        cumulative += weight; // Validated sum is at most 2**64.
        if u128::from(ticket) < cumulative {
            return Ok(index);
        }
    }
    unreachable!("validated ticket lies within cumulative sum")
}

pub fn weighted_index(
    state: &StreamState,
    weights: &[u128],
) -> Result<(usize, StreamState), RngError> {
    let total = weight_sum(weights)?;
    let (ticket, next) = uniform_below(state, total)?;
    Ok((weighted_from_ticket(weights, ticket)?, next))
}

pub(super) fn shuffle_from_words<T: Clone>(
    items: &[T],
    next: &mut impl FnMut() -> Result<u64, RngError>,
) -> Result<Vec<T>, RngError> {
    let mut result = items.to_vec();
    for i in (1..result.len()).rev() {
        let j = uniform_from_words(i as u128 + 1, next)? as usize;
        result.swap(i, j);
    }
    Ok(result)
}

pub fn shuffle<T: Clone>(
    state: &StreamState,
    items: &[T],
) -> Result<(Vec<T>, StreamState), RngError> {
    let mut cursor = state.clone();
    let result = shuffle_from_words(items, &mut || {
        let (word, next) = draw_u64(&cursor)?;
        cursor = next;
        Ok(word)
    })?;
    Ok((result, cursor))
}
