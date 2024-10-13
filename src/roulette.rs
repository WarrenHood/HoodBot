use std::{collections::BTreeMap, fmt::Display, hash::Hash, str::FromStr};
use anyhow::{Context, Result};
use rand::Rng;

pub fn is_red(number: u8) -> bool {
    // In number ranges from 1 to 10 and 19 to 28,
    //  odd numbers are red and even are black.
    // In ranges from 11 to 18 and 29 to 36,
    // odd numbers are black and even are red. 
    ((number % 2 == 1) && ((number >= 1 && number <= 10) || (number >= 19 && number <= 28))) ||
    ((number % 2 == 0) && ((number >= 11 && number <= 18) || (number >= 29 && number <= 36)))
}

pub fn is_black(number: u8) -> bool {
    number > 0 && !is_red(number)
}

#[derive(Debug, Clone, Copy)]
pub enum Bet {
    Single {
        number: u8
    },
    Red,
    Black,
    Even,
    Odd,
    Dozen{
        nth: u8
    }
}

impl Bet {
    pub fn get_payout_ratio(&self) -> u128 {
        match self {
            Bet::Single { number: _ } => 36,
            Bet::Red => 2,
            Bet::Black => 2,
            Bet::Even => 2,
            Bet::Odd => 2,
            Bet::Dozen { nth } => 3,
        }
    }

    pub fn is_correct(&self, spin_result: u8) -> bool {
        match *self {
            Bet::Single { number } => spin_result == number,
            Bet::Red => is_red(spin_result),
            Bet::Black => is_black(spin_result),
            Bet::Even => spin_result > 0 && spin_result % 2 == 0,
            Bet::Odd => spin_result > 0 && spin_result % 2 == 1,
            Bet::Dozen { nth } => spin_result != 0 && spin_result >= nth * 12 + 1 && spin_result <= (nth + 1) * 12,
        }
    }

    pub fn from_string(s: &str) -> Result<Vec<Self>> {
        let mut bets = vec![];

        let words: Vec<&str> = s.split(' ').collect();
        match words.first() {
            Some(first_word) => {
                match *first_word {
                    "single" => {
                        for word in words.iter().skip(1) {
                            let value = u8::from_str(word).context(format!("Unable to parse number for single bet: '{word}'"))?;
                            bets.push(Bet::Single { number: value });
                        }
                    },
                    "red" => bets.push(Bet::Red),
                    "black" => bets.push(Bet::Black),
                    "even" => bets.push(Bet::Even),
                    "odd" => bets.push(Bet::Odd),
                    "dozen1" => bets.push(Bet::Dozen { nth: 0 }),
                    "dozen2" => bets.push(Bet::Dozen { nth: 1 }),
                    "dozen3" => bets.push(Bet::Dozen { nth: 2 }),
                    _ => anyhow::bail!("Unrecognized bet type: '{first_word}'")
                }
            },
            None => anyhow::bail!("Invalid bet command: '{s}'"),
        };

        Ok(bets)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PlayerBet {
    bet: Bet,
    amount: u128
}

impl PlayerBet {
    pub fn get_payout(&self, spin_result: u8) -> u128 {
        if self.bet.is_correct(spin_result) {
            self.bet.get_payout_ratio() * self.amount
        }
        else {
            0
        }
    }

    pub fn from_string(s: &str) -> Result<Vec<Self>> {
        let words: Vec<&str> = s.split(' ').collect();
        if let Some(amount) = words.first() {
            let amount = u128::from_str(amount).context(format!("Invalid bet amount: '{amount}'. Expected an integer"))?;
            let rest: Vec<&str> = words.into_iter().skip(1).collect();
            let bet_string = rest.join(" ");
            let bets = Bet::from_string(&bet_string).context(format!("Failed to parse bet: '{bet_string}'"))?;
            return Ok(bets.into_iter().map(|bet| PlayerBet {
                bet,
                amount
            }).collect());
        }
        anyhow::bail!("Expected a bet amount");
    }
}

pub struct Player<T> {
    /// Uniqaue player identifier
    player_id: T,
    player_name: String,
    balance: u128,
    bets: Vec<PlayerBet>
}

impl<T> Player<T> where T: Display {
    pub fn new(player_id: T, player_name: &str) -> Self {
        Player { player_id, player_name: player_name.into(), balance: 1000, bets: Default::default() }
    }

    pub fn bet(&mut self, player_bet: PlayerBet) -> Result<()> {
        if player_bet.amount > self.balance {
            anyhow::bail!("Balance of {} is too low for bet {player_bet:#?}", self.balance);
        }
        if player_bet.amount == 0 {
            anyhow::bail!("You cannot place a bet with a value of zero!");
        }
        self.balance -= player_bet.amount;
        self.bets.push(player_bet);
        println!("Player {} (id={}) placed bet {player_bet:#?}", self.player_id, self.player_name);
        Ok(())
    }

    pub fn clear_last_bet(&mut self) {
        if let Some(last_bet) = self.bets.pop() {
            self.balance += last_bet.amount;
            println!("Player {} (id={}) undid last bet {last_bet:#?}", self.player_name, self.player_id);
        }
    }

    pub fn clear_all_bets(&mut self) {
        while !self.bets.is_empty() {
            self.clear_last_bet();
        }
    }
}

pub struct RouletteState<T> {
    players: BTreeMap<T, Player<T>>,
    can_change_bets: bool,
    pub spin_scheduled: bool
}

pub struct SpinResult {
    pub result: u8,
    pub payouts: BTreeMap<String, (u128, u128)>,
}

impl<T> RouletteState<T> where T: Display + Eq + Hash + Clone + Ord {
    pub fn new() -> Self {
        RouletteState {
            players: Default::default(),
            can_change_bets: true,
            spin_scheduled: false
        }
    }

    pub fn register_player(&mut self, id: T, name: &str) {
        if !self.players.contains_key(&id) {
            self.players.insert(id.clone(), Player::new(id, name));
        }
    }

    pub fn bet(&mut self, player_id: T, player_bet: PlayerBet) -> Result<()> {
        if let Some(player) = self.players.get_mut(&player_id) {
            if !self.can_change_bets {
                anyhow::bail!(
                    "Player {} (id={}) attempted to place bet {player_bet:#?} while bets were locked in", player.player_name, player_id
                );
            }
            player.bet(player_bet).context(format!("Couldn't place bet for player {} (id={player_id})", player.player_name))?;
        }
        else {
            anyhow::bail!("Player with id {player_id} is not registered to play roulette!")
        }
        Ok(())
    }

    pub fn play_bet_command(&mut self, player_id: T, bet_command: &str) -> Result<()> {
        let bets = PlayerBet::from_string(bet_command).context(format!("Unable to parse bet '{bet_command}'"))?;
        for bet in bets.into_iter() {
            self.bet(player_id.clone(), bet)?;
        }
        Ok(())
    }

    pub fn clear_last_bet(&mut self, player_id: T) -> Result<()> {
        if let Some(player) = self.players.get_mut(&player_id) {
            if !self.can_change_bets {
                anyhow::bail!(
                    "Player {} (id={}) attempted to clear last bet while bets were locked in", player.player_name, player_id
                );
            }
            player.clear_last_bet();
        }
        else {
            anyhow::bail!("Player with id {player_id} is not registered to play roulette!")
        }
        Ok(())
    }

    pub fn clear_all_bets(&mut self, player_id: T) -> Result<()> {
        if let Some(player) = self.players.get_mut(&player_id) {
            if !self.can_change_bets {
                anyhow::bail!(
                    "Player {} (id={}) attempted to clear all bets while bets were locked in", player.player_name, player_id
                );
            }
            player.clear_all_bets();
        }
        else {
            anyhow::bail!("Player with id {player_id} is not registered to play roulette!")
        }
        Ok(())
    }

    pub fn lock_bets(&mut self) {
        self.can_change_bets = false;
        println!("Bets are now locked in");
    }

    pub fn get_bets(&self, player_id: T) -> Result<Vec<PlayerBet>> {
        if let Some(player) = self.players.get(&player_id) {
            Ok(player.bets.clone())
        }
        else {
            anyhow::bail!("Player with id {player_id} is not registered to play roulette!")
        }
    }

    pub fn spin(&mut self) -> SpinResult {
        let mut thread_rng = rand::thread_rng();
        let mut payouts: BTreeMap<String, (u128, u128)> = BTreeMap::default();
        let result = thread_rng.gen_range(0..36) as u8;
        println!("Spin result: {result}");
        for player in self.players.values_mut() {
            let mut total_payout: u128 = 0;
            for player_bet in player.bets.iter() {
                total_payout += player_bet.get_payout(result);
            }
            if player.bets.len() > 0 {
                player.bets.clear();
                player.balance += total_payout;
                println!("Player {} (id={}) received payout of {total_payout}", player.player_name, player.player_id);
                let player_key = format!("{} (id={})", player.player_name, player.player_id);
                payouts.insert(player_key, (total_payout, player.balance));
            }
        }
        self.can_change_bets = true;
        self.spin_scheduled = false;
        SpinResult {
            result,
            payouts,
        }
    }

    pub fn get_balance(&self, player_id: T) -> Result<u128> {
        if let Some(player) = self.players.get(&player_id) {
            Ok(player.balance)
        }
        else {
            anyhow::bail!("Player with id {player_id} is not registered to play roulette!")
        }
    }

    pub fn set_balance(&mut self, player_id: T, balance: u128) -> Result<()> {
        if let Some(player) = self.players.get_mut(&player_id) {
            println!("Set player {} (id={player_id})'s balance to {balance}", player.player_name);
            player.balance = balance;
            Ok(())
        }
        else {
            anyhow::bail!("Player with id {player_id} is not registered to play roulette!")
        }
    }
}