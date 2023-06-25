use std::{collections::HashMap, io::stdout};

use itertools::Itertools;
use ordered_float::OrderedFloat;
use rand::Rng;
use serde::Serialize;

use crate::Outcome::*;

type Money = f32;
type RoundN = u16;

const NUM_SIMULATIONS: u32 = 1000;
const NUM_ROUNDS: RoundN = 1000;

#[derive(Clone, Debug, Serialize)]
enum Outcome {
    WIN,
    LOSS,
}

#[derive(Clone, Debug, Serialize)]
pub struct Event {
    bet: Money,
    outcome: Outcome,
    new_money: Money,
}

fn main() {
    let mut histories: Vec<(RoundN, Money, Vec<Event>)> = Vec::new();
    let initial_money: Money = 100.0;

    for _n in 0..NUM_SIMULATIONS {
        let mut history: Vec<Event> = Vec::new();
        let (round, money) = step(
            1,
            initial_money,
            |_, money| f32::min(money, 20.0),
            |bet| {
                if rand::thread_rng().gen::<bool>() {
                    (WIN, bet * 1.8)
                } else {
                    (LOSS, bet * 0.5)
                }
            },
            |round, money| money <= 1.0 || round >= NUM_ROUNDS,
            &mut history,
        );
        histories.push((round, money, history));
    }

    // serde_json::to_writer_pretty(stdout(), &histories).expect("serialization to console errored");

    histories.sort_by_cached_key(|t| OrderedFloat(t.1));
    let median = if histories.len() % 2 == 0 {
        (histories[histories.len() / 2 - 1].1 + histories[histories.len() / 2].1) / 2.0
    } else {
        histories[histories.len() / 2].1
    };
    let mode = histories
        .iter()
        .dedup_by_with_count(|t1, t2| t1.1 == t2.1)
        .sorted_by_cached_key(|t| t.0)
        .rev()
        .take(1)
        .next()
        .expect("empty iterations?")
        .1
         .1;

    println!(
        "Mean: {}, median: {}, mode: {}",
        histories
            .iter()
            .fold(0.0, |acc: Money, (_, money, _)| acc + money)
            / histories.len() as f32,
        median,
        mode
    );

    println!("top 10:");
    histories
        .iter()
        .rev()
        .take(10)
        .for_each(|(round, money, _)| println!("{:>5} | {:>20.2}", round, money));
}

// TODO fn simulate(numRuns, )

fn step(
    round: u16,
    money: f32,
    bet: fn(round: RoundN, money: Money) -> Money,
    play: fn(bet: Money) -> (Outcome, Money),
    should_stop: fn(RoundN, Money) -> bool,
    history: &mut Vec<Event>,
) -> (RoundN, Money) {
    if should_stop(round, money) {
        return (round, money);
    }

    let bet_amount = bet(round, money);
    assert!(bet_amount <= money);
    let (outcome, returns) = play(bet_amount);
    let new_money = money + returns;

    history.push(Event {
        bet: bet_amount,
        outcome,
        new_money,
    });
    step(round + 1, new_money, bet, play, should_stop, history)
}
