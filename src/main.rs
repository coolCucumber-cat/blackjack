use fmt2::write_to::WriteTo;

const PLAYER_COUNT: usize = 5;
const GAME_COUNT: usize = 1_000_000;
// const CARDS_IN_DECK: usize = CARDS_LIST.len() * 4;

macro_rules! declare_cards {
    {
        enum $Enum:ident {
            $($Variant:ident = $value:expr => $running_count_value:expr),* $(,)?
        }
        const $CARDS_LIST:ident;
        // const $CARDS_TOTAL:ident;
    } => {
        #[derive( Copy, Clone, PartialEq, Eq)]
        pub enum $Enum {
            $($Variant),*
        }

        impl $Enum {
            pub const fn value(self) -> u8 {
                match self {
                    $(Self::$Variant => $value),*
                }
            }

            pub const fn running_count_value(self) -> i32 {
                match self {
                    $(Self::$Variant => $running_count_value),*
                }
            }
        }

        const $CARDS_LIST: &[$Enum] = &[
            $($Enum::$Variant),*
        ];

        // const $CARDS_TOTAL: u8 = 0 $(+ $value)*;
    };
}

declare_cards! {
    enum Card {
        Two = 2 => 1,
        Three = 3 => 1,
        Four = 4 => 1,
        Five = 5 => 1,
        Six = 6 => 1,
        Seven = 7 => 0,
        Eight = 8 => 0,
        Nine = 9 => 0,
        Ten = 10 => -1,
        Jack = 10 => -1,
        Queen = 10 => -1,
        King = 10 => -1,
        Ace = 1 => -1,
    }
    const CARDS_LIST;
    // const CARDS_TOTAL;
}

pub struct Deck {
    pub cards: Vec<Card>,
    pub running_count_value: i32,
}

impl Deck {
    fn new() -> Self {
        use rand::seq::SliceRandom;

        let mut cards: Vec<Card> = CARDS_LIST
            .iter()
            .chain(CARDS_LIST.iter())
            .chain(CARDS_LIST.iter())
            .chain(CARDS_LIST.iter())
            .copied()
            .collect();
        // shuffle cards randomly
        cards.shuffle(&mut rand::thread_rng());

        Self {
            cards,
            running_count_value: 0,
        }
    }

    fn deal_card(&mut self, hand: &mut Hand) -> Card {
        let card = self.cards.pop().unwrap_or_else(|| {
            *self = Self::new();
            self.cards.pop().expect("unreachable")
        });
        self.running_count_value += card.running_count_value();
        hand.0.push(card);
        card
    }
}

pub struct Hand(Vec<Card>);

impl Hand {
    fn new() -> Self {
        Self(Vec::with_capacity(2))
    }

    fn value(&self) -> u8 {
        let (value_sum, ace_count) =
            self.0
                .iter()
                .fold((0_u8, 0_u8), |(acc_value_sum, acc_ace_count), &card| {
                    (
                        acc_value_sum + card.value(),
                        if card == Card::Ace {
                            acc_ace_count + 1
                        } else {
                            acc_ace_count
                        },
                    )
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Blackjack,
    Standing(u8),
    Bust,
}

pub enum Move {
    Stand,
    Hit,
    DoubleDown,
}

pub struct Money {
    betted: i32,
    won: i32,
}

impl Money {
    pub const fn profit(&self) -> i32 {
        self.won - self.betted
    }

    pub fn percent(&self) -> f64 {
        f64::from(self.won) / f64::from(self.betted)
    }
}

impl WriteTo for Money {
    fmt2::fmt! { [s] =>
        {s.betted} " => " {s.won} " = " {s.percent()} "%"
    }
}

pub fn blind_strategy(highest_card: u8, hand_value: u8) -> Move {
    match hand_value {
        9..=11 => Move::DoubleDown,
        hand_value if hand_value > highest_card => Move::Stand,
        _ => Move::Hit,
    }
}

pub fn basic_strategy(hand_value: u8, dealer_visible_card: Card) -> Move {
    match (hand_value, dealer_visible_card) {
        (9, Card::Three | Card::Four | Card::Five | Card::Six) => Move::DoubleDown,
        (
            10,
            Card::Two
            | Card::Three
            | Card::Four
            | Card::Five
            | Card::Six
            | Card::Seven
            | Card::Eight
            | Card::Nine,
        ) => Move::DoubleDown,
        (11, _) => Move::DoubleDown,
        (12, Card::Four | Card::Five | Card::Six) => Move::Stand,
        (13..=16, Card::Two | Card::Three | Card::Four | Card::Five | Card::Six) => Move::Stand,
        (0..=16, _) => Move::Hit,
        _ => Move::Stand,
    }
}

pub fn card_counting_betting_strategy(running_count: i32) -> u8 {
    (running_count - 1).try_into().unwrap_or(0)
}

pub fn card_counting_betting_strategy_agressive(running_count: i32) -> u8 {
    running_count.try_into().unwrap_or(0)
}

pub fn card_counting_betting_strategy_passive(running_count: i32) -> u8 {
    (running_count - 2).try_into().unwrap_or(0)
}

pub fn play_turn_with_strategy(
    dealer_visible_card: Card,
    deck: &mut Deck,
    player_hand: &mut Hand,
    first_turn: bool,
    mut doubled_down: bool,
    strategy: impl Fn(u8, Card) -> Move,
) -> (Outcome, bool) {
    match player_hand.value() {
        21 => (Outcome::Blackjack, doubled_down),
        value if value > 21 => (Outcome::Bust, doubled_down),
        value => {
            let player_move = strategy(value, dealer_visible_card);
            match player_move {
                Move::Stand => (Outcome::Standing(value), doubled_down),
                Move::Hit | Move::DoubleDown => {
                    if matches!(player_move, Move::DoubleDown) && first_turn {
                        doubled_down = true;
                    }
                    deck.deal_card(player_hand);
                    play_turn_with_strategy(
                        dealer_visible_card,
                        deck,
                        player_hand,
                        false,
                        doubled_down,
                        strategy,
                    )
                }
            }
        }
    }
}

pub fn play_dealer_turn(deck: &mut Deck, dealer_hand: &mut Hand) -> Outcome {
    match dealer_hand.value() {
        21 => Outcome::Blackjack,
        v if v > 21 => Outcome::Bust,
        v if v > 16 => Outcome::Standing(v),
        _ => {
            deck.deal_card(dealer_hand);
            play_dealer_turn(deck, dealer_hand)
        }
    }
}

pub fn play_game(
    strategy: &impl Fn(u8, Card) -> Move,
    betting_strategy: impl Fn(i32) -> u8,
) -> Money {
    let mut money = Money { betted: 0, won: 0 };
    let mut deck = Deck::new();
    for _ in 0..GAME_COUNT {
        let bet = i32::from(betting_strategy(deck.running_count_value));
        let mut dealer_hand = Hand::new();
        let mut player_hands: [Hand; PLAYER_COUNT] = [
            Hand::new(),
            Hand::new(),
            Hand::new(),
            Hand::new(),
            Hand::new(),
        ];
        // let mut player_hands: Vec<Hand> = (0..PLAYER_COUNT).map(|_| Hand::new()).collect();

        // deal out 2 cards
        for player_hand in player_hands.iter_mut() {
            deck.deal_card(player_hand);
        }
        let dealer_visible_card = deck.deal_card(&mut dealer_hand);
        for player_hand in player_hands.iter_mut() {
            deck.deal_card(player_hand);
        }
        deck.deal_card(&mut dealer_hand);

        let dealer_outcome = play_dealer_turn(&mut deck, &mut dealer_hand);
        for player_hand in player_hands.iter_mut() {
            let (outcome, double_down) = play_turn_with_strategy(
                dealer_visible_card,
                &mut deck,
                player_hand,
                true,
                false,
                strategy,
            );
            let mut money_betted = bet * 2;
            if double_down {
                money_betted *= 2;
            }
            money.betted += money_betted;
            match (outcome, dealer_outcome) {
                (Outcome::Blackjack, _) => money.won += (money_betted * 3) / 2,
                (Outcome::Standing(_), Outcome::Bust) => money.won += money_betted * 2,
                (Outcome::Standing(v), Outcome::Standing(d)) if v > d => {
                    money.won += money_betted * 2
                }
                (Outcome::Standing(v), Outcome::Standing(d)) if v == d => {
                    money.won += money_betted;
                }
                _ => {}
            }
        }
    }
    money
}

fn main() {
    let mut stdout = std::io::stdout().lock();
    fmt2::fmt! { (stdout) =>
        "GAME_COUNT = " {GAME_COUNT} ln
        "PLAYER_COUNT = " {PLAYER_COUNT} ln
        ln
    };

    for highest_card in 11..=16 {
        let money = play_game(
            &|hand_value, _| blind_strategy(highest_card, hand_value),
            |_| 2,
        );
        fmt2::fmt! { (stdout) =>
            "blind strategy (höchster wert = " {highest_card} ")" ln
            "   " {money} ln
        }
    }
    let money = play_game(&basic_strategy, |_| 2);
    fmt2::fmt! { (stdout) =>
        "basic strategy" ln
        "   " {money} ln
    }

    let money = play_game(&basic_strategy, card_counting_betting_strategy);
    fmt2::fmt! { (stdout) =>
        "card counting" ln
        "   " {money} ln
    }

    let money = play_game(&basic_strategy, card_counting_betting_strategy_passive);
    fmt2::fmt! { (stdout) =>
        "card counting (passive)" ln
        "   " {money} ln
    }

    let money = play_game(&basic_strategy, card_counting_betting_strategy_agressive);
    fmt2::fmt! { (stdout) =>
        "card counting (agressive)" ln
        "   " {money} ln
    }
}
