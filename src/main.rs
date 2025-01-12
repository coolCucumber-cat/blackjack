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

pub trait Value {
    fn value(&self) -> u8;
}

impl Value for [Card] {
    fn value(&self) -> u8 {
        let (value_sum, ace_count) =
            self.iter()
                .fold((0_u8, 0_u8), |(acc_value_sum, acc_ace_count), &card| {
                    match CardNoAce::try_from(card) {
                        Ok(card_not_ace) => (acc_value_sum + card_not_ace.value(), acc_ace_count),
                        Err(()) => (acc_value_sum + 1, acc_ace_count + 1),
                    }
                });
        let Some(value_sum_until_21) = 21_u8.checked_sub(value_sum) else {
            return value_sum;
        };
        let ace_count_needed_for_max_value = value_sum_until_21 / 10;
        let actual_ace_count = ace_count_needed_for_max_value.min(ace_count);
        let aces_value_sum = actual_ace_count * 10;
        aces_value_sum + value_sum
    }
}

fmt2::enum_alias! {
    enum CardNoAce: Card = {
        Two |
        Three |
        Four |
        Five |
        Six |
        Seven |
        Eight |
        Nine |
        Ten |
        Jack |
        Queen |
        King
    };
}

impl CardNoAce {
    pub const fn value(self) -> u8 {
        match self {
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
            Self::Five => 5,
            Self::Six => 6,
            Self::Seven => 7,
            Self::Eight => 8,
            Self::Nine => 9,
            Self::Ten => 10,
            Self::Jack => 10,
            Self::Queen => 10,
            Self::King => 10,
        }
    }
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

fn take_turn(
    highest_card: u8,
    deck: &mut Vec<Card>,
    hand: &mut Vec<Card>,
) -> UndecidedPlayerOutcome {
    match hand.value() {
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
