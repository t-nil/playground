use std::{
    collections::VecDeque,
    fmt::Debug,
    ops::{Deref, DerefMut},
    vec,
};

use itertools::Itertools;

#[derive(Debug, Clone, Copy)]
struct Process {
    arrival: usize,
    computation_time: usize,
}

impl Process {
    fn new(arrival: usize, computation_time: usize) -> Self {
        Process {
            arrival,
            computation_time,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RealtimeProcess {
    computation_time: usize,
    period_length: usize,
}

#[derive(Debug, Default)]
struct Schedule(Vec<Option<usize>>);
impl Schedule {
    fn new() -> Self {
        Schedule(vec![])
    }
}

impl Deref for Schedule {
    type Target = Vec<Option<usize>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Schedule {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// trait Scheduler<T = Process> = Fn(impl Iterator<Item = T>) -> Schedule;
// trait RealtimeScheduler = Scheduler<RealtimeProcess>;

// fn round_robin2(quantum: usize) -> impl Scheduler {
//     |ps: |
// }

//trait Scheduler = impl Fn(Iterator<Item = Process>) -> Schedule;

fn round_robin(ps: impl Iterator<Item = Process>, quantum: usize) -> Schedule {
    let mut ps = ps.enumerate().collect_vec();
    ps.sort_by_cached_key(|p| p.1.arrival);

    let mut queue: VecDeque<(usize, Process)> = VecDeque::new();
    let mut round = 0;
    let mut time_spent_on_cur = 0;
    let mut schedule: Schedule = Schedule::new();
    while !(ps.is_empty() && queue.is_empty()) {
        // take all arriving processes and put them into the queue
        ps.drain_filter(|p| p.1.arrival <= round)
            .collect_into(&mut queue);
        assert!(round + 1 > schedule.len());
        schedule.resize_with(round + 1, || None);

        if let Some(mut cur_proc) = queue.pop_front() {
            assert!(cur_proc.1.computation_time > 0);

            // pulled the quantum check to the front, now needlessly inserting once front and pulling to back immediately after
            // but now newly arriving tasks get correctly scheduled before cycled tasks
            if (time_spent_on_cur >= quantum) {
                queue.push_back(cur_proc);
                cur_proc = queue.pop_front().expect("I just pushed sth, didn't I");
                time_spent_on_cur = 0;
            }

            schedule[round] = Some(cur_proc.0);
            cur_proc.1.computation_time -= 1;
            time_spent_on_cur += 1;

            if cur_proc.1.computation_time > 0 {
                queue.push_front(cur_proc);
            } else {
                time_spent_on_cur = 0;
            }
        }

        round += 1;
    }

    schedule
}

pub fn test_round_robin() {
    println!("\n ## ROUND ROBIN");

    println!("\n### irgendwas\n");

    let v = vec![
        Process::new(0, 4),
        Process::new(2, 3),
        Process::new(4, 6),
        Process::new(11, 3),
        Process::new(12, 6),
    ];
    let quantum = 3;

    let schedule = round_robin(v.clone().into_iter(), quantum);
    schedule_to_text_diagram(&v, schedule);

    println!("\n### Altklausur SS15\n");

    let v = vec![
        Process::new(1, 6),
        Process::new(4, 2),
        Process::new(2, 4),
        Process::new(9, 3),
        Process::new(8, 4),
    ];
    let quantum = 3;
    let schedule = round_robin(v.clone().into_iter(), quantum);
    schedule_to_text_diagram(&v, schedule);

    println!("\n### Altklausur SS18\n");

    let v = vec![
        Process::new(9, 3),
        Process::new(8, 4),
        Process::new(4, 4),
        Process::new(1, 6),
        Process::new(2, 5),
    ];
    let quantum = 3;
    let schedule = round_robin(v.clone().into_iter(), quantum);
    schedule_to_text_diagram(&v, schedule);
}

fn rate_monotonic(ps: impl Iterator<Item = RealtimeProcess>) -> Schedule {
    let mut ps = ps.enumerate().collect_vec();
    ps.sort_by_cached_key(|p| p.1.period_length);

    let tmax = ps
        .iter()
        .map(|p| p.1.period_length)
        .reduce(num_integer::lcm)
        .expect(format!("Could not calculate LCM from {:?}", ps).as_str());

    let mut cycles_done: Vec<usize> = vec![0; ps.len()];
    let mut result: Schedule = Schedule(Vec::new());
    'outer: for round in 0..tmax {
        for i in 0..ps.len() {
            if round % ps[i].1.period_length == 0 {
                cycles_done[i] = ps[i].1.computation_time;
            }
        }

        for i in 0..ps.len() {
            if cycles_done[i] > 0 {
                result.push(Some(ps[i].0));
                cycles_done[i] -= 1;
                continue 'outer;
            }
        }

        result.push(None);
    }

    result
}

pub fn test_rate_monotonic() {
    println!("\n## RATE MONOTONIC\n");

    println!("\n### Altklausur SS15:");
    let processes = vec![
        RealtimeProcess {
            computation_time: 2,
            period_length: 10,
        },
        RealtimeProcess {
            computation_time: 1,
            period_length: 5,
        },
        RealtimeProcess {
            computation_time: 5,
            period_length: 20,
        },
    ];

    // TODO borrow, not move
    let schedule = rate_monotonic(processes.clone().into_iter());
    schedule_to_text_diagram(&processes, schedule);

    println!("\n### Altklausur SS15:");

    let processes = vec![
        RealtimeProcess {
            computation_time: 1,
            period_length: 6,
        },
        RealtimeProcess {
            computation_time: 1,
            period_length: 3,
        },
        RealtimeProcess {
            computation_time: 3,
            period_length: 18,
        },
        RealtimeProcess {
            computation_time: 2,
            period_length: 9,
        },
    ];

    // TODO borrow, not move
    let schedule = rate_monotonic(processes.clone().into_iter());
    schedule_to_text_diagram(&processes, schedule);
}

fn schedule_to_text_diagram(ps: &[impl Debug], s: Schedule) {
    //println!("processes: {:#?}, schedule: {:#?}", ps, s);

    ps.iter().enumerate().for_each(|(i, _)| {
        // TODO maybe reduce to_owned
        let mut counter = 0;
        println!(
            "Process {}: {}",
            i,
            s.iter()
                .map(|x| if *x == Some(i) {
                    // TODO reset counter on period switch, like Stapperts Vorgabe
                    counter += 1;
                    (counter % 10).to_string()
                } else {
                    " ".to_owned()
                })
                .enumerate()
                .map(|(i, c)| if i % 5 == 0 {
                    "|".to_owned() + &c
                } else {
                    c.to_owned()
                })
                .collect::<String>()
        );
    });
    // print scale
    let legend = format!("Round/5  : {}", "|     ".repeat(s.len() / 5));
    println!("{}", "-".repeat(legend.len()));
    println!("{}", legend);
}
