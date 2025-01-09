#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Card {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UndecidedPlayerOutcome {
    Won,
    Undecided(u8),
    Lost,
}

const CARDS_ONCE: &[Card; 13] = &[
    Card::Two,
    Card::Three,
    Card::Four,
    Card::Five,
    Card::Six,
    Card::Seven,
    Card::Eight,
    Card::Nine,
    Card::Ten,
    Card::Jack,
    Card::Queen,
    Card::King,
    Card::Ace,
];

const PLAYER_COUNT: usize = 5;

fn take_card_from_deck(deck: &mut Vec<Card>, player_cards: &mut Vec<Card>) {
    player_cards.push(deck.pop().expect("ran out of cards"));
}

fn get_cards_value(cards: &[Card]) -> u8 {
    cards.iter().fold(0, |value, &card| {
        value
            + match card {
                Card::Two => 2,
                Card::Three => 3,
                Card::Four => 4,
                Card::Five => 5,
                Card::Six => 6,
                Card::Seven => 7,
                Card::Eight => 8,
                Card::Nine => 9,
                Card::Ten => 10,
                Card::Jack => 10,
                Card::Queen => 10,
                Card::King => 10,
                Card::Ace => 1,
            }
    })
    // let aces = cards.iter().filter(|&&card| card == Card::Ace);
    // let no_aces = cards.iter().filter(|&&card| card != Card::Ace);
    // let aces_count = aces.cloned().count();
    // no_aces.fold(0, |value, &card| {
    //     value
    //         + match card {
    //             Card::Two => 2,
    //             Card::Three => 3,
    //             Card::Four => 4,
    //             Card::Five => 5,
    //             Card::Six => 6,
    //             Card::Seven => 7,
    //             Card::Eight => 8,
    //             Card::Nine => 9,
    //             Card::Ten => 10,
    //             Card::Jack => 10,
    //             Card::Queen => 110,
    //             Card::King => 10,
    //             Card::Ace => unreachable!("filtered out all aces, aces are unreachable"),
    //         }
    // })+aces.enumerate().fold(0, |value, (i, card)|)
}

fn take_turn(
    highest_card: u8,
    deck: &mut Vec<Card>,
    hand: &mut Vec<Card>,
) -> UndecidedPlayerOutcome {
    match get_cards_value(hand) {
        21 => UndecidedPlayerOutcome::Won,
        v if v > 21 => UndecidedPlayerOutcome::Lost,
        v if v > highest_card => UndecidedPlayerOutcome::Undecided(v),
        _ => {
            take_card_from_deck(deck, hand);
            take_turn(highest_card, deck, hand)
        }
    }
}

fn main() {
    use rand::seq::SliceRandom;

    let mut stdout = std::io::stdout().lock();
    for highest_card in 2..=20 {
        let mut wins: u32 = 0;
        let mut draws: u32 = 0;
        let mut losses: u32 = 0;

        for _ in 0..0x4FFFF_u32 {
            // create deck of cards with 13 * 4 cards
            let mut deck: Vec<Card> = CARDS_ONCE
                .iter()
                .chain(CARDS_ONCE.iter())
                .chain(CARDS_ONCE.iter())
                .chain(CARDS_ONCE.iter())
                .copied()
                .collect();
            // shuffle cards randomly
            deck.shuffle(&mut rand::thread_rng());

            let mut dealer_hand = Vec::with_capacity(3);
            let mut player_hands: Vec<Vec<Card>> =
                (0..PLAYER_COUNT).map(|_| Vec::with_capacity(3)).collect();

            // deal out 2 cards
            for _ in 0..2 {
                for hand in player_hands.iter_mut() {
                    take_card_from_deck(&mut deck, hand);
                }
                take_card_from_deck(&mut deck, &mut dealer_hand);
            }

            let dealer_outcome = take_turn(16, &mut deck, &mut dealer_hand);
            let player_outcomes = player_hands
                .iter_mut()
                .map(|hand| take_turn(highest_card, &mut deck, hand));
            match dealer_outcome {
                UndecidedPlayerOutcome::Won => {
                    for outcome in player_outcomes {
                        if outcome == UndecidedPlayerOutcome::Won {
                            wins += 1;
                        } else {
                            losses += 1;
                        }
                    }
                }
                UndecidedPlayerOutcome::Lost => {
                    for outcome in player_outcomes {
                        if outcome != UndecidedPlayerOutcome::Lost {
                            wins += 1;
                        } else {
                            losses += 1;
                        }
                    }
                }
                UndecidedPlayerOutcome::Undecided(dealer_value) => {
                    for outcome in player_outcomes {
                        match outcome {
                            UndecidedPlayerOutcome::Won => wins += 1,
                            UndecidedPlayerOutcome::Undecided(player_value) => {
                                match player_value.cmp(&dealer_value) {
                                    core::cmp::Ordering::Greater => wins += 1,
                                    core::cmp::Ordering::Equal => draws += 1,
                                    core::cmp::Ordering::Less => losses += 1,
                                }
                            }
                            UndecidedPlayerOutcome::Lost => losses += 1,
                        }
                    }
                }
            }
        }
        let wins_f64 = f64::from(wins);
        let wins_losses_f64 = f64::from(wins + losses);
        let win_percent = wins_f64 / wins_losses_f64;
        fmt2::fmt! { (stdout) =>
            {highest_card} ln
            "   gewonnen: " {wins} " hex " {wins;h} ln
            "   unentsch: " {draws} " hex " {draws;h} ln
            "   verloren: " {losses} " hex " {losses;h} ln
            "          %: " {win_percent} ln
        };
    }
}
