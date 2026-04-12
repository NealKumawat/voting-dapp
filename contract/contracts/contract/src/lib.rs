#![no_std]

use soroban_sdk::{
    contract, contractevent, contractimpl, contracttype,
    Address, Env, String, Vec,
};

// ── Data Types ────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Donation {
    pub donor:     Address,
    pub amount:    i128,       // in stroops (1 XLM = 10_000_000 stroops)
    pub message:   String,
    pub timestamp: u64,
    pub id:        u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FundUsage {
    pub description: String,
    pub amount:      i128,
    pub timestamp:   u64,
    pub reported_by: Address,
    pub id:          u64,
}

#[contracttype]
pub enum DataKey {
    Admin,
    TotalRaised,
    TotalSpent,
    DonationCount,
    UsageCount,
    Donation(u64),
    Usage(u64),
    DonorHistory(Address),
}

// ── Events ────────────────────────────────────────────────────────────────────

#[contractevent]
pub struct Initialized {
    pub admin: Address,
}

#[contractevent]
pub struct Donated {
    pub donor: Address,
    pub id:    u64,
    pub amount: i128,
}

#[contractevent]
pub struct FundsSpent {
    pub reported_by: Address,
    pub id:          u64,
    pub amount:      i128,
}

#[contractevent]
pub struct AdminTransferred {
    pub new_admin: Address,
}

// ── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct DonationTracker;

#[contractimpl]
impl DonationTracker {

    // ── Initialization ────────────────────────────────────────────────────────

    /// Initialize the contract with an admin address.
    /// Can only be called once.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TotalRaised, &0_i128);
        env.storage().instance().set(&DataKey::TotalSpent,  &0_i128);
        env.storage().instance().set(&DataKey::DonationCount, &0_u64);
        env.storage().instance().set(&DataKey::UsageCount,    &0_u64);

        env.events().publish_event(&Initialized { admin });
    }

    // ── Donations ─────────────────────────────────────────────────────────────

    /// Record a donation from a donor.
    /// `amount` is in stroops. `message` is an optional note (pass "" for none).
    pub fn donate(env: Env, donor: Address, amount: i128, message: String) -> u64 {
        donor.require_auth();

        if amount <= 0 {
            panic!("donation amount must be positive");
        }

        let count: u64 = env.storage().instance().get(&DataKey::DonationCount).unwrap_or(0);
        let id = count + 1;

        let donation = Donation {
            donor:     donor.clone(),
            amount,
            message,
            timestamp: env.ledger().timestamp(),
            id,
        };

        // Store the donation record
        env.storage().persistent().set(&DataKey::Donation(id), &donation);

        // Update donor history (list of their donation IDs)
        let mut history: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::DonorHistory(donor.clone()))
            .unwrap_or(Vec::new(&env));
        history.push_back(id);
        env.storage().persistent().set(&DataKey::DonorHistory(donor.clone()), &history);

        // Update totals
        let raised: i128 = env.storage().instance().get(&DataKey::TotalRaised).unwrap_or(0);
        env.storage().instance().set(&DataKey::TotalRaised, &(raised + amount));
        env.storage().instance().set(&DataKey::DonationCount, &id);

        env.events().publish_event(&Donated { donor, id, amount });

        id
    }

    // ── Fund Usage ────────────────────────────────────────────────────────────

    /// Admin reports how funds were used (transparent spending record).
    pub fn report_usage(
        env:         Env,
        description: String,
        amount:      i128,
    ) -> u64 {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized");
        admin.require_auth();

        if amount <= 0 {
            panic!("usage amount must be positive");
        }

        let spent: i128  = env.storage().instance().get(&DataKey::TotalSpent).unwrap_or(0);
        let raised: i128 = env.storage().instance().get(&DataKey::TotalRaised).unwrap_or(0);

        if spent + amount > raised {
            panic!("reported spend exceeds total raised");
        }

        let count: u64 = env.storage().instance().get(&DataKey::UsageCount).unwrap_or(0);
        let id = count + 1;

        let usage = FundUsage {
            description,
            amount,
            timestamp:   env.ledger().timestamp(),
            reported_by: admin.clone(),
            id,
        };

        env.storage().persistent().set(&DataKey::Usage(id), &usage);
        env.storage().instance().set(&DataKey::TotalSpent,  &(spent + amount));
        env.storage().instance().set(&DataKey::UsageCount,  &id);

        env.events().publish_event(&FundsSpent { reported_by: admin, id, amount });

        id
    }

    // ── Read-Only Queries ─────────────────────────────────────────────────────

    /// Get a single donation record by its ID.
    pub fn get_donation(env: Env, id: u64) -> Donation {
        env.storage()
            .persistent()
            .get(&DataKey::Donation(id))
            .expect("donation not found")
    }

    /// Get a single fund-usage record by its ID.
    pub fn get_usage(env: Env, id: u64) -> FundUsage {
        env.storage()
            .persistent()
            .get(&DataKey::Usage(id))
            .expect("usage record not found")
    }

    /// Fetch all donation IDs made by a specific donor.
    pub fn get_donor_history(env: Env, donor: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::DonorHistory(donor))
            .unwrap_or(Vec::new(&env))
    }

    /// Total funds raised (sum of all donations, in stroops).
    pub fn total_raised(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::TotalRaised).unwrap_or(0)
    }

    /// Total funds spent as reported by admin (in stroops).
    pub fn total_spent(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::TotalSpent).unwrap_or(0)
    }

    /// Remaining/unallocated funds (raised − spent), in stroops.
    pub fn remaining_funds(env: Env) -> i128 {
        let raised: i128 = env.storage().instance().get(&DataKey::TotalRaised).unwrap_or(0);
        let spent:  i128 = env.storage().instance().get(&DataKey::TotalSpent).unwrap_or(0);
        raised - spent
    }

    /// Total number of donations recorded.
    pub fn donation_count(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::DonationCount).unwrap_or(0)
    }

    /// Total number of fund-usage reports recorded.
    pub fn usage_count(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::UsageCount).unwrap_or(0)
    }

    /// Return the contract admin address.
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized")
    }

    // ── Admin Utilities ───────────────────────────────────────────────────────

    /// Transfer admin rights to a new address.
    pub fn transfer_admin(env: Env, new_admin: Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized");
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &new_admin);

        env.events().publish_event(&AdminTransferred { new_admin });
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::{Address as _, Ledger}, Env, String};

    fn setup() -> (Env, DonationTrackerClient<'static>, Address, Address) {
        let env    = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, DonationTracker);
        let client      = DonationTrackerClient::new(&env, &contract_id);
        let admin       = Address::generate(&env);
        let donor       = Address::generate(&env);
        client.initialize(&admin);
        (env, client, admin, donor)
    }

    #[test]
    fn test_initialize() {
        let (_, client, admin, _) = setup();
        assert_eq!(client.get_admin(), admin);
        assert_eq!(client.total_raised(), 0);
        assert_eq!(client.total_spent(),  0);
    }

    #[test]
    fn test_donate_and_query() {
        let (env, client, _, donor) = setup();
        let msg = String::from_str(&env, "For the kids!");
        let id  = client.donate(&donor, &5_000_000_i128, &msg);
        assert_eq!(id, 1);
        assert_eq!(client.total_raised(), 5_000_000);
        assert_eq!(client.donation_count(), 1);

        let d = client.get_donation(&1);
        assert_eq!(d.donor,  donor);
        assert_eq!(d.amount, 5_000_000);
    }

    #[test]
    fn test_multiple_donations_same_donor() {
        let (env, client, _, donor) = setup();
        let msg = String::from_str(&env, "");
        client.donate(&donor, &1_000_000_i128, &msg);
        client.donate(&donor, &2_000_000_i128, &msg);
        assert_eq!(client.total_raised(), 3_000_000);
        let hist = client.get_donor_history(&donor);
        assert_eq!(hist.len(), 2);
    }

    #[test]
    fn test_report_usage() {
        let (env, client, admin, donor) = setup();
        let msg  = String::from_str(&env, "");
        let desc = String::from_str(&env, "Bought medical supplies");
        client.donate(&donor, &10_000_000_i128, &msg);
        let id = client.report_usage(&desc, &3_000_000_i128);
        assert_eq!(id, 1);
        assert_eq!(client.total_spent(), 3_000_000);
        assert_eq!(client.remaining_funds(), 7_000_000);
    }

    #[test]
    #[should_panic(expected = "donation amount must be positive")]
    fn test_zero_donation_rejected() {
        let (env, client, _, donor) = setup();
        let msg = String::from_str(&env, "");
        client.donate(&donor, &0_i128, &msg);
    }

    #[test]
    #[should_panic(expected = "reported spend exceeds total raised")]
    fn test_overspend_rejected() {
        let (env, client, _, donor) = setup();
        let msg  = String::from_str(&env, "");
        let desc = String::from_str(&env, "Too much");
        client.donate(&donor, &1_000_000_i128, &msg);
        client.report_usage(&desc, &5_000_000_i128);
    }

    #[test]
    fn test_transfer_admin() {
        let (env, client, _, donor) = setup();
        client.transfer_admin(&donor);
        assert_eq!(client.get_admin(), donor);
    }
}