/// Binary operators
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    // Arithmetic
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Mod, // %
    Pow, // **

    // String
    Concat, // .

    // Comparison
    Equal,        // ==
    Identical,    // ===
    NotEqual,     // !=
    NotIdentical, // !==
    LessThan,     // <
    GreaterThan,  // >
    LessEqual,    // <=
    GreaterEqual, // >=
    Spaceship,    // <=>

    // Logical
    And, // && or 'and'
    Or,  // || or 'or'
    Xor, // xor

    // Null coalescing
    NullCoalesce, // ??

    // Pipe operator
    Pipe, // |> (PHP 8.5)
}

/// Unary operators
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,     // -
    Not,     // !
    PreInc,  // ++$x
    PreDec,  // --$x
    PostInc, // $x++
    PostDec, // $x--
}

/// Assignment operators
#[derive(Debug, Clone, PartialEq)]
pub enum AssignOp {
    Assign,       // =
    AddAssign,    // +=
    SubAssign,    // -=
    MulAssign,    // *=
    DivAssign,    // /=
    ModAssign,    // %=
    ConcatAssign, // .=
}
