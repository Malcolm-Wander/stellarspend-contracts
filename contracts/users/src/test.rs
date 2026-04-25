use soroban_sdk::{testutils::Address as _, Address, Env};
use crate::{UsersContract, UserError};

#[test]
fn test_initialize_and_get_admin() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(UsersContract, ());
    
    // Test initialization
    UsersContract::initialize(env.clone(), admin.clone());
    
    // Verify admin is set
    assert_eq!(UsersContract::get_admin(env.clone()), Some(admin.clone()));
    
    // Test duplicate initialization fails
    let admin2 = Address::generate(&env);
    let result = std::panic::catch_unwind(|| {
        UsersContract::initialize(env.clone(), admin2);
    });
    assert!(result.is_err());
}

#[test]
fn test_register_user_and_count() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(UsersContract, ());
    
    UsersContract::initialize(env.clone(), admin.clone());
    
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);
    
    // Test initial count is 0
    assert_eq!(UsersContract::get_all_users_count(env.clone()), 0);
    
    // Register first user
    let is_new1 = UsersContract::register_user(env.clone(), user1.clone());
    assert!(is_new1);
    assert_eq!(UsersContract::get_all_users_count(env.clone()), 1);
    assert!(UsersContract::is_user_registered(env.clone(), user1.clone()));
    
    // Register second user
    let is_new2 = UsersContract::register_user(env.clone(), user2.clone());
    assert!(is_new2);
    assert_eq!(UsersContract::get_all_users_count(env.clone()), 2);
    assert!(UsersContract::is_user_registered(env.clone(), user2.clone()));
    
    // Register third user
    let is_new3 = UsersContract::register_user(env.clone(), user3.clone());
    assert!(is_new3);
    assert_eq!(UsersContract::get_all_users_count(env.clone()), 3);
    assert!(UsersContract::is_user_registered(env.clone(), user3.clone()));
    
    // Test duplicate registration (should not increase count)
    let is_duplicate = UsersContract::register_user(env.clone(), user1.clone());
    assert!(!is_duplicate);
    assert_eq!(UsersContract::get_all_users_count(env.clone()), 3);
}

#[test]
fn test_get_all_users_admin_only() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let contract_id = env.register(UsersContract, ());
    
    UsersContract::initialize(env.clone(), admin.clone());
    
    // Register some users
    UsersContract::register_user(env.clone(), user.clone());
    
    // Test admin can get all users
    let all_users = UsersContract::get_all_users(env.clone(), admin.clone());
    assert_eq!(all_users.len(), 1);
    assert_eq!(all_users.get(0), user);
    
    // Test non-admin cannot get all users
    let result = std::panic::catch_unwind(|| {
        UsersContract::get_all_users(env.clone(), user.clone());
    });
    assert!(result.is_err());
}

#[test]
fn test_user_exists_functionality() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(UsersContract, ());
    
    UsersContract::initialize(env.clone(), admin.clone());
    
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    
    // Test non-existent user
    assert!(!UsersContract::is_user_registered(env.clone(), user1.clone()));
    assert!(!UsersContract::is_user_registered(env.clone(), user2.clone()));
    
    // Register user1
    UsersContract::register_user(env.clone(), user1.clone());
    
    // Test user1 exists, user2 doesn't
    assert!(UsersContract::is_user_registered(env.clone(), user1.clone()));
    assert!(!UsersContract::is_user_registered(env.clone(), user2.clone()));
    
    // Register user2
    UsersContract::register_user(env.clone(), user2.clone());
    
    // Test both users exist
    assert!(UsersContract::is_user_registered(env.clone(), user1.clone()));
    assert!(UsersContract::is_user_registered(env.clone(), user2.clone()));
}

// ── Issue #336: check_user_exists ────────────────────────────────────────────

#[test]
fn test_check_user_exists_returns_false_for_unregistered() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let _contract_id = env.register(UsersContract, ());

    UsersContract::initialize(env.clone(), admin.clone());

    let stranger = Address::generate(&env);
    assert!(!UsersContract::check_user_exists(env.clone(), stranger));
}

#[test]
fn test_check_user_exists_returns_true_after_registration() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let _contract_id = env.register(UsersContract, ());

    UsersContract::initialize(env.clone(), admin.clone());

    let user = Address::generate(&env);
    // Before registration → false
    assert!(!UsersContract::check_user_exists(env.clone(), user.clone()));

    // After registration → true
    UsersContract::register_user(env.clone(), user.clone());
    assert!(UsersContract::check_user_exists(env.clone(), user.clone()));
}

#[test]
fn test_check_user_exists_matches_is_user_registered() {
    // check_user_exists is a deliberate alias for is_user_registered.
    // This test enforces parity so any future divergence is caught.
    let env = Env::default();
    let admin = Address::generate(&env);
    let _contract_id = env.register(UsersContract, ());

    UsersContract::initialize(env.clone(), admin.clone());

    let registered = Address::generate(&env);
    let unregistered = Address::generate(&env);

    UsersContract::register_user(env.clone(), registered.clone());

    // Both functions must agree on a registered user
    assert_eq!(
        UsersContract::check_user_exists(env.clone(), registered.clone()),
        UsersContract::is_user_registered(env.clone(), registered.clone()),
    );

    // Both functions must agree on an unregistered user
    assert_eq!(
        UsersContract::check_user_exists(env.clone(), unregistered.clone()),
        UsersContract::is_user_registered(env.clone(), unregistered.clone()),
    );
}

#[test]
fn test_multiple_unique_users() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(UsersContract, ());
    
    UsersContract::initialize(env.clone(), admin.clone());
    
    let mut users = Vec::new(&env);
    
    // Create and register 10 unique users
    for i in 0..10 {
        let user = Address::generate(&env);
        users.push_back(user.clone());
        UsersContract::register_user(env.clone(), user);
    }
    
    // Verify count matches
    assert_eq!(UsersContract::get_all_users_count(env.clone()), 10);
    
    // Verify all users are registered
    for i in 0..10 {
        assert!(UsersContract::is_user_registered(env.clone(), users.get(i)));
    }
}
