#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;

use alloy_primitives::{ Address, U8, I8, U256, I32, U32 };
use stylus_sdk::{ prelude::*, msg, block };

use stylus_sdk::call::Call;

// this is the time overlay for each vote, that causes stakers to not claim reward (spam claim);
const TIMEDELAY: u32 = 5 * 60;

sol_storage! {
    #[entrypoint]
    pub struct Main{
        // this will be mapping the gallary inmdex with the 
        // mapping of the nfts with status; so the data will be transfered to the safe mapping
        mapping(uint256 => Concept) gallary_data;
    }

    pub struct Nft{
        string name;
        string meta_data;
        // this status will be as follows
        // 0 undergoing review
        // 1 accepted 
        // 3 rejected
        uint8 status;
        address owner;
    }

    pub struct Concept{
        mapping(uint256 => Nft) data_x;
        uint256[] accepted;
        uint256 available_index;
    }
}

sol! {
    event AcceptedNft(address indexed creator, uint256 gallary, uint256 nft_index);
    event RegetedNft(address indexed creator, uint256 gallary, uint256 nft_index);


    // my error
    error InvalidParameter(uint8 point);
    error DeniedAccess(uint256 Nft_index);
}

#[public]
impl RewardState {
    pub fn set_erc2o_address(&mut self, address: Address) {
        self.erc20.set(address);
    }

    pub fn submit_nft(&mut self, nft_name: String, nft_meta_data: String, gallary_index: U256) {
        // so we will run a function here that will check if the user has bought the ticket fro the gallary
        // so the function that you see here is just the after effet of the action

        // run function that will check if the gallary exist and run another that will check if the user is allowed to view to participate inthe gallary

        let mut gallary = self.gallary_data.setter(gallary_index);
        let available_index = gallary.available_index.get();

        let system_data = gallary.data_x.setter();
        gallary.available_index.set(available_index + U256::from(1));
    }

    pub fn get_reward(&mut self, content_id: u8) {
        if self.is_rewarded(content_id) {
            return;
        }

        if !self.can_be_rewarded(content_id) {
            return;
        }

        let total_votes = self.content_vote.get(U8::from(content_id)).total_votes.get();
        if total_votes >= I32::unchecked_from(2) {
            self.reward(content_id, 1);
        } else if total_votes <= I32::unchecked_from(-2) {
            self.reward(content_id, 0);
        }
    }

    pub fn is_rewarded(&self, content_id: u8) -> bool {
        let voter_y = msg::sender();
        let voter_x = self.voters.get(U8::from(content_id));
        for inx in 0..voter_x.len() {
            let voter = voter_x.get(inx).unwrap();
            if voter.user_id.get() == voter_y {
                return voter.rewarded.get();
            }
        }
        return true;
    }

    pub fn can_be_rewarded(&self, content_id: u8) -> bool {
        let current_time_stamp = U32::from(block::timestamp());
        let old_time = self.content_vote.get(U8::from(content_id)).time_stamp.get();
        let over_lap_time = U32::from(TIMEDELAY) + old_time;
        if current_time_stamp >= over_lap_time {
            return true;
        }
        false
    }

    pub fn my_vote(&self, content_id: u8) -> String {
        let voter_y = msg::sender();
        let mut result = String::from("0");
        let voter_x = self.voters.get(U8::from(content_id));
        for inx in 0..voter_x.len() {
            let voter = voter_x.get(inx).unwrap();
            if voter.user_id.get() == voter_y {
                let my_vote_x = voter.vote.get();
                result = format!("{}", my_vote_x);
            }
        }
        return result;
    }
}

impl RewardState {
    pub fn reward(&mut self, content_id: u8, winner: u8) {
        let mut content = self.voters.setter(U8::from(content_id));
        let mut reward_data = U256::from(0);

        for index in 0..content.len() {
            let mut vote_x = content.get_mut(index).unwrap();
            let address_re = vote_x.user_id.get();

            if msg::sender() == address_re {
                let stake = vote_x.stake.get();
                vote_x.rewarded.set(true);

                let (multiplier, addition) = if
                    (winner == 0 && vote_x.vote.get() == I8::unchecked_from(-1)) || // Losing vote case
                    (winner == 1 && vote_x.vote.get() == I8::unchecked_from(1)) // Winning vote case
                {
                    (12, true)
                } else {
                    (7, false)
                };

                let rd = stake * U256::from(multiplier);
                let re = rd / U256::from(100);
                reward_data = if addition {
                    stake + re
                } else {
                    if re > stake { U256::from(0) } else { stake - re }
                };

                break;
            }
        }

        self.trf_vote_reward(reward_data, msg::sender());
    }

    pub fn trf_vote_reward(&mut self, reward: U256, address: Address) {
        let meta_date_contract = IErc20::new(*self.erc20);
        let config = Call::new_in(self);
        let _ = meta_date_contract
            .transfer(config, address, reward)
            .expect("Failed to call on MetaDate_contract");
    }
}
