/// Enumeration used to filter move generation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoveFilter {
    /// Generate all moves
    All,
    /// Generate "tactical" moves - those that change material balance
    /// including promotions.
    Tacticals,
    /// Generate captures only (no quiet promotions, only captures)
    Captures,
    /// Generate quiets only (no captures, EP, or promotions)
    Quiets,
}
