/// The two card families in Kessel Sabacc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Family {
    /// Sand cards, represented in amber/gold.
    Sand,
    /// Blood cards, represented in red.
    Blood,
}

/// The value of a card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardValue {
    /// A numbered card with value 1-6.
    Number(u8),
    /// Sylop: takes the value of the other card in hand (effectively 0 difference).
    Sylop,
    /// Impostor: value determined by dice roll at revelation.
    Impostor,
}

/// A single Sabacc card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    /// Which family this card belongs to.
    pub family: Family,
    /// The value of this card.
    pub value: CardValue,
}

impl Card {
    /// Create a new numbered card.
    pub fn number(family: Family, n: u8) -> Self {
        Self {
            family,
            value: CardValue::Number(n),
        }
    }

    /// Create a new Sylop card.
    pub fn sylop(family: Family) -> Self {
        Self {
            family,
            value: CardValue::Sylop,
        }
    }

    /// Create a new Impostor card.
    pub fn impostor(family: Family) -> Self {
        Self {
            family,
            value: CardValue::Impostor,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_constructors() {
        let sand3 = Card::number(Family::Sand, 3);
        assert_eq!(sand3.family, Family::Sand);
        assert_eq!(sand3.value, CardValue::Number(3));

        let blood_sylop = Card::sylop(Family::Blood);
        assert_eq!(blood_sylop.family, Family::Blood);
        assert_eq!(blood_sylop.value, CardValue::Sylop);

        let sand_impostor = Card::impostor(Family::Sand);
        assert_eq!(sand_impostor.family, Family::Sand);
        assert_eq!(sand_impostor.value, CardValue::Impostor);
    }

    #[test]
    fn card_clone_and_eq() {
        let card = Card::number(Family::Blood, 5);
        let cloned = card.clone();
        assert_eq!(card, cloned);
    }
}
