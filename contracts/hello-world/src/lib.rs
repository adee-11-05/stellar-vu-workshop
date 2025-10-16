#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, Address};

#[contracttype]
#[derive(Clone)]
pub struct UserWallet {
    pub user_address: Address,
    pub total_achievements: u64,
    pub total_rewards: u64,
    pub last_claim_time: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct Achievement {
    pub achievement_id: u64,
    pub name: String,
    pub reward_points: u64,
}

#[contracttype]
pub enum WalletBook {
    Wallet(Address),
}

#[contracttype]
pub enum AchievementBook {
    Achievement(u64),
}

const ACHIEVEMENT_COUNT: Symbol = Symbol::short("ACH_CNT");

#[contract]
pub struct RewardWalletContract;

#[contractimpl]
impl RewardWalletContract {
    pub fn create_achievement(env: Env, name: String, reward_points: u64) -> u64 {
        let mut storage = env.storage().instance();

        let mut achievement_count: u64 = storage.get(&ACHIEVEMENT_COUNT).unwrap_or(0);
        achievement_count += 1;

        let achievement = Achievement {
            achievement_id: achievement_count,
            name: name.clone(),
            reward_points,
        };

        storage.set(&AchievementBook::Achievement(achievement_count), &achievement);
        storage.set(&ACHIEVEMENT_COUNT, &achievement_count);

        // Extend TTL for this key
        storage.extend_ttl(&AchievementBook::Achievement(achievement_count), 5000);

        log!(&env, "Achievement created with ID: {}", achievement_count);
        achievement_count
    }

    pub fn award_achievement(env: Env, user: Address, achievement_id: u64) {
        user.require_auth();

        let achievement = Self::view_achievement(env.clone(), achievement_id);

        if achievement.achievement_id == 0 {
            log!(&env, "Achievement not found!");
            panic!("Achievement not found!");
        }

        let mut wallet = Self::view_wallet(env.clone(), user.clone());
        let time = env.ledger().timestamp();

        wallet.user_address = user.clone();
        wallet.total_achievements += 1;
        wallet.total_rewards += achievement.reward_points;
        wallet.last_claim_time = time;

        let mut storage = env.storage().instance();
        storage.set(&WalletBook::Wallet(user.clone()), &wallet);
        storage.extend_ttl(&WalletBook::Wallet(user.clone()), 5000);

        log!(
            &env,
            "Achievement awarded to user. Total rewards: {}",
            wallet.total_rewards
        );
    }

    pub fn view_wallet(env: Env, user: Address) -> UserWallet {
        let key = WalletBook::Wallet(user.clone());
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or(UserWallet {
                user_address: user,
                total_achievements: 0,
                total_rewards: 0,
                last_claim_time: 0,
            })
    }

    pub fn view_achievement(env: Env, achievement_id: u64) -> Achievement {
        let key = AchievementBook::Achievement(achievement_id);
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or(Achievement {
                achievement_id: 0,
                name: String::from_str(&env, "Not_Found"),
                reward_points: 0,
            })
    }
}
