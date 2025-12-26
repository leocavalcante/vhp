//! Operator precedence levels for Pratt parsing

use crate::token::TokenKind;

/// Operator precedence levels (higher = binds tighter)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    None = 0,
    Assignment = 1,    // = += -= etc.
    Ternary = 2,       // ?:
    NullCoalesce = 3,  // ??
    Or = 4,            // || or
    And = 5,           // && and
    Xor = 6,           // xor
    Equality = 7,      // == === != !==
    Comparison = 8,    // < > <= >= <=>
    Concat = 9,        // .
    AddSub = 10,       // + -
    MulDiv = 11,       // * / %
    Pow = 12,          // ** (right associative)
    Unary = 13,        // ! - ++ --
}

/// Get precedence for a token kind
pub fn get_precedence(kind: &TokenKind) -> Precedence {
    match kind {
        TokenKind::Assign
        | TokenKind::PlusAssign
        | TokenKind::MinusAssign
        | TokenKind::MulAssign
        | TokenKind::DivAssign
        | TokenKind::ModAssign
        | TokenKind::ConcatAssign => Precedence::Assignment,

        TokenKind::QuestionMark => Precedence::Ternary,
        TokenKind::NullCoalesce => Precedence::NullCoalesce,

        TokenKind::Or => Precedence::Or,
        TokenKind::And => Precedence::And,
        TokenKind::Xor => Precedence::Xor,

        TokenKind::Equal
        | TokenKind::Identical
        | TokenKind::NotEqual
        | TokenKind::NotIdentical => Precedence::Equality,

        TokenKind::LessThan
        | TokenKind::GreaterThan
        | TokenKind::LessEqual
        | TokenKind::GreaterEqual
        | TokenKind::Spaceship => Precedence::Comparison,

        TokenKind::Concat => Precedence::Concat,
        TokenKind::Plus | TokenKind::Minus => Precedence::AddSub,
        TokenKind::Mul | TokenKind::Div | TokenKind::Mod => Precedence::MulDiv,
        TokenKind::Pow => Precedence::Pow,

        _ => Precedence::None,
    }
}

/// Check if operator is right-associative
pub fn is_right_assoc(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Pow
            | TokenKind::Assign
            | TokenKind::PlusAssign
            | TokenKind::MinusAssign
            | TokenKind::MulAssign
            | TokenKind::DivAssign
            | TokenKind::ModAssign
            | TokenKind::ConcatAssign
            | TokenKind::NullCoalesce
    )
}
