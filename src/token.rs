/// Token types for the VHP lexer
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // PHP Tags
    OpenTag,  // <?php
    CloseTag, // ?>

    // Keywords
    Echo,     // echo
    True,     // true
    False,    // false
    Null,     // null
    If,       // if
    Else,     // else
    Elseif,   // elseif
    While,    // while
    For,      // for
    Foreach,  // foreach
    As,       // as
    Switch,   // switch
    Case,     // case
    Default,  // default
    Break,    // break
    Continue, // continue
    Do,       // do

    // Alternative syntax end keywords
    Endif,      // endif
    Endwhile,   // endwhile
    Endfor,     // endfor
    Endforeach, // endforeach
    Endswitch,  // endswitch
    Function, // function
    Fn,       // fn (arrow function, PHP 7.4)
    Return,   // return
    Match,    // match (PHP 8.0)

    // OOP Keywords
    Class,      // class
    New,        // new
    Public,     // public
    Private,    // private
    Protected,  // protected
    Extends,    // extends
    Parent,     // parent
    Interface,  // interface
    Implements, // implements
    Trait,      // trait
    Use,        // use (for traits in class and namespace imports)
    Insteadof,  // insteadof
    Readonly,   // readonly (PHP 8.1)
    Enum,       // enum (PHP 8.1)
    Clone,      // clone (PHP 5.0)
    Fiber,      // fiber (PHP 8.1)
    With,       // with (PHP 8.4) - for clone with syntax
    Abstract,   // abstract (for abstract classes and methods)
    Final,      // final (for final classes, methods, and constants)
    Static,     // static (for static methods and properties)
    Get,        // get (PHP 8.4) - property hooks
    Set,        // set (PHP 8.4) - property hooks

    // Namespace Keywords
    Namespace,  // namespace
    Const,      // const (for use const and class constants)

    // Exception Keywords
    Try,     // try
    Catch,   // catch
    Finally, // finally
    Throw,   // throw

    // Identifiers and Variables
    Variable(String),   // $name
    Identifier(String), // function names, etc.

    // Literals
    String(String), // "string" or 'string'
    Integer(i64),   // 123
    Float(f64),     // 1.23

    // Assignment Operators
    Assign,       // =
    PlusAssign,   // +=
    MinusAssign,  // -=
    MulAssign,    // *=
    DivAssign,    // /=
    ModAssign,    // %=
    ConcatAssign, // .=

    // Arithmetic Operators
    Plus,  // +
    Minus, // -
    Mul,   // *
    Div,   // /
    Mod,   // %
    Pow,   // **

    // String Operator
    Concat, // .

    // Comparison Operators
    Equal,        // ==
    Identical,    // ===
    NotEqual,     // !=
    NotIdentical, // !==
    LessThan,     // <
    GreaterThan,  // >
    LessEqual,    // <=
    GreaterEqual, // >=
    Spaceship,    // <=>

    // Logical Operators
    And, // && or 'and'
    Or,  // || or 'or'
    Not, // !
    Xor, // xor

    // Bitwise Operators
    BitwiseOr, // | (used in multi-catch and bitwise operations)

    // Increment/Decrement
    Increment, // ++
    Decrement, // --

    // Punctuation
    Semicolon,    // ;
    Comma,        // ,
    LeftParen,    // (
    RightParen,   // )
    LeftBrace,    // {
    RightBrace,   // }
    LeftBracket,  // [
    RightBracket, // ]
    QuestionMark, // ?
    Colon,        // :
    NullCoalesce, // ??
    DoubleArrow,  // =>
    Arrow,        // ->
    DoubleColon,  // ::
    Pipe,         // |> (PHP 8.5 pipe operator)
    Hash,         // # (for attributes when followed by [)
    Ellipsis,     // ... (variadic/spread operator)
    Backslash,    // \ (for namespaces and fully qualified names)

    // Special
    Html(String), // Raw HTML outside PHP tags
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
