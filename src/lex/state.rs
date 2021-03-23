use std::fmt::Display;


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub enum State {
    Initial,
    PrepareComment,
    PrepareAssignment,
    SingleQuote,
    DoubleQuote,
    Integer,
    IntegerDecimal,
    Identifier,
    PrepareOpenTag,
    PrepareCloseTag,
    LineComment,
    MultiLineComment,
    MultiLineCommentPrepareExit,
    Decimal,
    SingleQuoteEscape,
    DoubleQuoteEscape,
    DoubleQuoteEscapeControl,
    DoubleQuoteEscapeHex,
    DoubleQuoteEscapeOctal2,
    DoubleQuoteEscapeHexBound,
    DoubleQuoteEscapeHex2,
    PHPTag0,
    PHPTag1,
    PHPTag2,
    DoubleQuoteEscapeOctal3,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub struct Position {
    pub index: usize,
    pub line: usize,
    pub column: usize,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Line {}, Column {}", self.line, self.column)
    }
}


impl Position {
    pub fn advance(&mut self, new_line: bool) {
        self.index += 1;
        if new_line {
            self.line += 1;
            self.column = 0;
        } else {
            self.column += 1;
        }
    }
}