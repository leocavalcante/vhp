/// Token types for the VHP lexer
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // PHP Tags
    OpenTag,      // <?php
    CloseTag,     // ?>

    // Keywords
    Echo,         // echo
    True,         // true
    False,        // false
    Null,         // null

    // Identifiers and Variables
    Variable(String),  // $name
    Identifier(String), // function names, etc.

    // Literals
    String(String),    // "string" or 'string'
    Integer(i64),      // 123
    Float(f64),        // 1.23

    // Assignment Operators
    Assign,            // =
    PlusAssign,        // +=
    MinusAssign,       // -=
    MulAssign,         // *=
    DivAssign,         // /=
    ModAssign,         // %=
    ConcatAssign,      // .=

    // Arithmetic Operators
    Plus,              // +
    Minus,             // -
    Mul,               // *
    Div,               // /
    Mod,               // %
    Pow,               // **

    // String Operator
    Concat,            // .

    // Comparison Operators
    Equal,             // ==
    Identical,         // ===
    NotEqual,          // !=
    NotIdentical,      // !==
    LessThan,          // <
    GreaterThan,       // >
    LessEqual,         // <=
    GreaterEqual,      // >=
    Spaceship,         // <=>

    // Logical Operators
    And,               // && or 'and'
    Or,                // || or 'or'
    Not,               // !
    Xor,               // xor

    // Increment/Decrement
    Increment,         // ++
    Decrement,         // --

    // Punctuation
    Semicolon,         // ;
    Comma,             // ,
    LeftParen,         // (
    RightParen,        // )
    LeftBrace,         // {
    RightBrace,        // }
    LeftBracket,       // [
    RightBracket,      // ]
    QuestionMark,      // ?
    Colon,             // :
    NullCoalesce,      // ??

    // Special
    Html(String),      // Raw HTML outside PHP tags
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, column: usize) -> Self {
        Self { kind, line, column }
    }
}
