use soroban_sdk::{Env, IntoVal, Val};

/// Shared validation errors for simple reusable helpers.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ValidationError {
    NegativeAmount,
    InvalidAddress,
    Overflow,
}

/// Validates that an amount is not negative.
pub fn validate_amount(amount: i128) -> Result<(), ValidationError> {
    if amount < 0 {
        Err(ValidationError::NegativeAmount)
    } else {
        Ok(())
    }
}

/// Validates a user address string format.
///
/// Accepts classic Stellar-style prefixes (`G` account, `C` contract)
/// and only base32 characters (`A-Z`, `2-7`).
pub fn validate_user_address(address: &soroban_sdk::String) -> Result<(), ValidationError> {
    if address.len() == 0 {
        return Err(ValidationError::InvalidAddress);
    }

    let bytes = address.to_bytes();
    let prefix = bytes.get(0).unwrap_or(0);
    if prefix != b'G' && prefix != b'C' {
        return Err(ValidationError::InvalidAddress);
    }

    for b in bytes.iter() {
        let is_upper_alpha = b >= b'A' && b <= b'Z';
        let is_base32_digit = b >= b'2' && b <= b'7';
        if !is_upper_alpha && !is_base32_digit {
            return Err(ValidationError::InvalidAddress);
        }
    }

    Ok(())
}

/// Read a counter from persistent storage without modifying it.
/// Returns 0 if the key does not exist.
pub fn get_counter<K>(env: &Env, counter_key: &K) -> u64
where
    K: IntoVal<Env, Val>,
{
    env.storage()
        .persistent()
        .get(counter_key)
        .unwrap_or(0)
}

/// Read a counter from instance storage without modifying it.
/// Returns 0 if the key does not exist.
pub fn get_counter_instance<K>(env: &Env, counter_key: &K) -> u64
where
    K: IntoVal<Env, Val>,
{
    env.storage()
        .instance()
        .get(counter_key)
        .unwrap_or(0)
}

/// Increment a counter in persistent storage by 1 and return the new value.
///
/// If the key does not exist, the counter starts at 0 and becomes 1.
/// Returns `Err(ValidationError::Overflow)` on arithmetic overflow.
pub fn increment_counter<K>(env: &Env, counter_key: &K) -> Result<u64, ValidationError>
where
    K: IntoVal<Env, Val>,
{
    increment_counter_by(env, counter_key, 1)
}

/// Increment a counter in persistent storage by `step` and return the new value.
///
/// If the key does not exist, the counter starts at 0.
/// Returns `Err(ValidationError::Overflow)` on arithmetic overflow.
pub fn increment_counter_by<K>(env: &Env, counter_key: &K, step: u64) -> Result<u64, ValidationError>
where
    K: IntoVal<Env, Val>,
{
    let current: u64 = env
        .storage()
        .persistent()
        .get(counter_key)
        .unwrap_or(0);

    let new_val = current.checked_add(step).ok_or(ValidationError::Overflow)?;
    env.storage()
        .persistent()
        .set(counter_key, &new_val);

    Ok(new_val)
}

/// Increment a counter in instance storage by 1 and return the new value.
///
/// If the key does not exist, the counter starts at 0 and becomes 1.
/// Returns `Err(ValidationError::Overflow)` on arithmetic overflow.
pub fn increment_counter_instance<K>(env: &Env, counter_key: &K) -> Result<u64, ValidationError>
where
    K: IntoVal<Env, Val>,
{
    increment_counter_instance_by(env, counter_key, 1)
}

/// Increment a counter in instance storage by `step` and return the new value.
///
/// If the key does not exist, the counter starts at 0.
/// Returns `Err(ValidationError::Overflow)` on arithmetic overflow.
pub fn increment_counter_instance_by<K>(env: &Env, counter_key: &K, step: u64) -> Result<u64, ValidationError>
where
    K: IntoVal<Env, Val>,
{
    let current: u64 = env
        .storage()
        .instance()
        .get(counter_key)
        .unwrap_or(0);

    let new_val = current.checked_add(step).ok_or(ValidationError::Overflow)?;
    env.storage()
        .instance()
        .set(counter_key, &new_val);

    Ok(new_val)
}

/// Reset a counter in persistent storage to 0.
pub fn reset_counter<K>(env: &Env, counter_key: &K)
where
    K: IntoVal<Env, Val>,
{
    env.storage().persistent().set(counter_key, &0u64);
}

/// Reset a counter in instance storage to 0.
pub fn reset_counter_instance<K>(env: &Env, counter_key: &K)
where
    K: IntoVal<Env, Val>,
{
    env.storage().instance().set(counter_key, &0u64);
}

#[cfg(test)]
mod tests {
    use super::{validate_amount, validate_user_address, ValidationError,
        increment_counter, increment_counter_by,
        increment_counter_instance, increment_counter_instance_by,
        get_counter, get_counter_instance,
        reset_counter, reset_counter_instance};
    use soroban_sdk::{Env, Symbol, String};

    #[test]
    fn accepts_zero_and_positive_amounts() {
        assert_eq!(validate_amount(0), Ok(()));
        assert_eq!(validate_amount(1), Ok(()));
        assert_eq!(validate_amount(1_000_000), Ok(()));
    }

    #[test]
    fn rejects_negative_amounts() {
        assert_eq!(validate_amount(-1), Err(ValidationError::NegativeAmount));
        assert_eq!(validate_amount(-99), Err(ValidationError::NegativeAmount));
    }

    #[test]
    fn accepts_valid_user_address() {
        let env = Env::default();
        let address = String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
        assert_eq!(validate_user_address(&address), Ok(()));
    }

    #[test]
    fn rejects_invalid_user_address() {
        let env = Env::default();

        let empty = String::from_str(&env, "");
        assert_eq!(validate_user_address(&empty), Err(ValidationError::InvalidAddress));

        let bad_prefix = String::from_str(&env, "XAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
        assert_eq!(validate_user_address(&bad_prefix), Err(ValidationError::InvalidAddress));

        let bad_chars = String::from_str(&env, "GINVALID!ADDRESS");
        assert_eq!(validate_user_address(&bad_chars), Err(ValidationError::InvalidAddress));
    }

    #[test]
    fn increment_counter_works() {
        let env = Env::default();
        let counter_key = Symbol::new(&env, "test_counter");

        // First increment should return 1
        assert_eq!(increment_counter(&env, &counter_key), Ok(1));

        // Second increment should return 2
        assert_eq!(increment_counter(&env, &counter_key), Ok(2));

        // Third increment should return 3
        assert_eq!(increment_counter(&env, &counter_key), Ok(3));
    }

    #[test]
    fn increment_counter_by_works() {
        let env = Env::default();
        let counter_key = Symbol::new(&env, "step_counter");

        // Increment by 5
        assert_eq!(increment_counter_by(&env, &counter_key, 5), Ok(5));

        // Increment by 10 more
        assert_eq!(increment_counter_by(&env, &counter_key, 10), Ok(15));
    }

    #[test]
    fn increment_counter_overflow() {
        let env = Env::default();
        let counter_key = Symbol::new(&env, "overflow_counter");

        // Set counter to max
        env.storage().persistent().set(&counter_key, &u64::MAX);

        // Incrementing should overflow
        assert_eq!(increment_counter(&env, &counter_key), Err(ValidationError::Overflow));
    }

    #[test]
    fn get_counter_works() {
        let env = Env::default();
        let counter_key = Symbol::new(&env, "read_counter");

        // Default is 0
        assert_eq!(get_counter(&env, &counter_key), 0);

        // After incrementing, get should reflect the new value
        increment_counter(&env, &counter_key).unwrap();
        assert_eq!(get_counter(&env, &counter_key), 1);
    }

    #[test]
    fn reset_counter_works() {
        let env = Env::default();
        let counter_key = Symbol::new(&env, "reset_counter");

        // Increment a few times
        increment_counter(&env, &counter_key).unwrap();
        increment_counter(&env, &counter_key).unwrap();
        assert_eq!(get_counter(&env, &counter_key), 2);

        // Reset
        reset_counter(&env, &counter_key);
        assert_eq!(get_counter(&env, &counter_key), 0);
    }

    #[test]
    fn instance_counter_works() {
        let env = Env::default();
        let counter_key = Symbol::new(&env, "inst_counter");

        // Instance storage increment
        assert_eq!(increment_counter_instance(&env, &counter_key), Ok(1));
        assert_eq!(increment_counter_instance(&env, &counter_key), Ok(2));

        // Instance storage get
        assert_eq!(get_counter_instance(&env, &counter_key), 2);

        // Instance storage increment by step
        assert_eq!(increment_counter_instance_by(&env, &counter_key, 10), Ok(12));

        // Instance storage reset
        reset_counter_instance(&env, &counter_key);
        assert_eq!(get_counter_instance(&env, &counter_key), 0);
    }
}
