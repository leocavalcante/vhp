//! Operator precedence levels for Pratt parsing

use crate::token::TokenKind;

/// Operator precedence levels (higher = binds tighter)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)]
pub enum Precedence {
    None = 0,
    Assignment = 1,    // = += -= etc.
    Pipe = 2,          // |> (pipe operator - PHP 8.5)
    Ternary = 3,       // ?:
    NullCoalesce = 4,  // ??
    Or = 5,            // || or
    And = 6,           // && and
    Xor = 7,           // xor
    Equality = 8,      // == === != !==
    Comparison = 9,    // < > <= >= <=>
    Concat = 10,       // .
    AddSub = 11,       // + -
    MulDiv = 12,       // * / %
    Pow = 13,          // ** (right associative)
    Unary = 14,        // ! - ++ --
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
        TokenKind::Pipe => Precedence::Pipe,
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
