use std::collections::HashMap;
use std::fmt::{self};

#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {

    // Keywords
    Inputs,
    Outputs,
    In,
    Out,
 
    // Punctuation
    Comma,
    ParenOpen,
    ParenClose,
    Newline,
    EOF,

    // Gates
    And,
    Or,
    Not,
    Nand,
    Nor,
    Xor,
    Xnor,

    // Subcircuits
    Subcircuit,
    End,

    // Identifier
    Identifier(String),
}

/// The main lexer struct that handles tokenization of source code
#[derive(Debug)]
pub struct Lexer {
    source: String,             // The complete source code to be tokenized
    position: usize,            // Current position in the source code
    line: usize,                // Current line number (for error reporting)
    column: usize,              // Current column number (for error reporting)
    current_char: Option<char>, // Current character being processed
}

/// Represents a location in the source code for error reporting
#[derive(Debug, Clone, Copy)]
pub struct Location {
    pub line: usize,        // Line number (1-based)
    pub column: usize,      // Column number (1-based)
}

impl Location {
    /// Creates a new Location instance
    fn new(line: usize, column: usize) -> Self {
        Location { line, column } 
    }
}

impl Lexer {
    /// Creates a new Lexer instance with the given source code
    pub fn new(source: String) -> Self {
        let mut lexer = Lexer {
            source,
            position: 0,
            line: 1,
            column: 1,
            current_char: None,
        };
        lexer.current_char = lexer.source.chars().nth(0);
        lexer
    }

    /// Returns the current location in the source code
    fn get_location(&self) -> Location {
        Location::new(self.line, self.column)
    }

    /// Advances the lexer position by one character
    /// Updates line and column numbers appropriately
    fn advance(&mut self) {
        if let Some(c) = self.current_char {
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        self.position += 1;
        self.current_char = self.source.chars().nth(self.position);
    }

    /// Skips over whitespace characters in the source code
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char {
            if c == '\n' {
                break; // Don't skip newlines
            } else if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(c) = self.current_char {
            if c == '\n' {
                self.advance();
                break;
            } else {
                self.advance();
            }
        }
    }
    /// Main tokenization function that returns the next token from the source
    /// Returns Result<TokenKind, LexerError> to handle potential errors
    pub fn get_next_token(&mut self) -> Result<TokenKind, LexerError> {
        self.skip_whitespace();
        let location = self.get_location();

        if self.current_char.is_none() {
            return Ok(TokenKind::EOF);
        }

        match self.current_char {
            None => Ok(TokenKind::EOF),
            Some(c) => match c {
                c if c.is_alphabetic() => Ok(self.identifier()?),
                '#' => {
                    self.skip_line_comment();
                    self.get_next_token()
                },
                ',' => { self.advance(); Ok(TokenKind::Comma) },
                '(' => { self.advance(); Ok(TokenKind::ParenOpen) },
                ')' => { self.advance(); Ok(TokenKind::ParenClose) },
                '\n' => { self.advance(); Ok(TokenKind::Newline) },
                _ => Err(LexerError::UnexpectedCharacter(c, location)),
            }
        }
    }

    /// Processes identifiers and keywords
    /// Returns either a keyword token or an identifier token
    pub fn identifier(&mut self) -> Result<TokenKind, LexerError> {
        let location = self.get_location();
        let mut id_str = String::new();

        while let Some(c) = self.current_char {
            if c.is_alphanumeric() || c == '_' {
                id_str.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if id_str.is_empty() {
            return Err(LexerError::InvalidIdentifier(id_str, location));
        }

        // Match against known keywords, return Identifier if not a keyword
        Ok(match id_str.to_uppercase().as_str() {
            "INPUTS" => TokenKind::Inputs,
            "OUTPUTS" => TokenKind::Outputs,
            "IN" => TokenKind::In,
            "OUT" => TokenKind::Out,
            "AND" => TokenKind::And,
            "OR" => TokenKind::Or,
            "NOT" => TokenKind::Not,
            "NAND" => TokenKind::Nand,
            "NOR" => TokenKind::Nor,
            "XOR" => TokenKind::Xor,
            "XNOR" => TokenKind::Xnor,
            "SUBCIRCUIT" => TokenKind::Subcircuit,
            "END" => TokenKind::End,
            _ => TokenKind::Identifier(id_str),
        })
    }
}

/// Represents possible errors that can occur during lexical analysis 
#[derive(Debug)]
pub enum LexerError {
    UnexpectedCharacter(char, Location),
    InvalidIdentifier(String, Location),
}

impl std::error::Error for LexerError {}

/// Implements Display trait for LexerError to provide human-readable error messages
impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexerError::UnexpectedCharacter(c, loc) => 
                write!(f, "Unexpected character '{}' at line {}, column {}", c, loc.line, loc.column),
            LexerError::InvalidIdentifier(id, loc) => 
                write!(f, "Invalid identifier '{}' at line {}, column {}", id, loc.line, loc.column),
        }
    }
}

#[derive(Debug)]
pub struct Subcircuit {
    pub name: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub components: Vec<Component>,
}

#[derive(Debug)]
pub struct Program {
    pub subcircuits: HashMap<String, Subcircuit>,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub components: Vec<Component>,
}

#[derive(Debug)]
pub struct Component {
    pub gate_type: GateType,
    pub identifier: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(Debug)]
pub enum GateType {
    And,
    Or,
    Not,
    Nand,
    Nor,
    Xor,
    Xnor,
    Subcircuit(String),
}

pub struct Parser {
    tokens: Vec<TokenKind>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<TokenKind>) -> Self {
        Parser { tokens, position: 0 }
    }

    fn current_token(&self) -> Option<&TokenKind> {
        self.tokens.get(self.position)
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn expect(&mut self, expected: TokenKind) -> Result<(), String> {
        if let Some(token) = self.current_token() {
            if *token == expected {
                self.advance();
                Ok(())
            } else {
                Err(format!("Expected {:?}, found {:?}", expected, token))
            }
        } else {
            Err(format!("Unexpected end of input, expected {:?}", expected))
        }
    }
}

impl Parser {
    pub fn parse_program(&mut self) -> Result<Program, String> {

        // Parse subcircuits first
        let mut subcircuits = HashMap::new();
        
        while let Some(token) = self.current_token() {
            match token {
                TokenKind::Subcircuit => {
                    let subcircuit = self.parse_subcircuit()?;
                    subcircuits.insert(subcircuit.name.clone(), subcircuit);
                }
                _ => break // Break if it's not a subcircuit
            }
        }

        let inputs = self.parse_inputs_section()?;
        let outputs = self.parse_outputs_section()?;
        let components = self.parse_component_list()?;
        Ok(Program { subcircuits, inputs, outputs, components })
    }
}

impl Parser {
    fn parse_subcircuit(&mut self) -> Result<Subcircuit, String> {
        self.expect(TokenKind::Subcircuit)?;
        
        let name = if let Some(TokenKind::Identifier(name)) = self.current_token() {
            name.clone()
        } else {
            return Err("Expected subcircuit name".to_string());
        };
        self.advance();
        
        self.expect(TokenKind::Newline)?;
        
        let inputs = self.parse_inputs_section()?;
        
        let outputs = self.parse_outputs_section()?;
        
        let components = self.parse_component_list()?;
        
        self.expect(TokenKind::End)?;
        
        self.expect(TokenKind::Newline)?;
        
        Ok(Subcircuit {
            name,
            inputs,
            outputs,
            components,
        })
    }
}

impl Parser {
    fn parse_inputs_section(&mut self) -> Result<Vec<String>, String> {

        loop {
            if self.current_token() == Some(&TokenKind::Newline) {
                self.advance();
            } else {
               break;
            }
        }

        self.expect(TokenKind::Inputs)?; // Expect "INPUTS"
        let mut inputs = vec![];

        loop {
            match self.current_token() {
                Some(TokenKind::Identifier(name)) => {
                    inputs.push(name.clone());
                    self.advance();
                }
                Some(TokenKind::Comma) => self.advance(),
                Some(TokenKind::Newline) => {
                    self.advance();
                    break;
                }
                _ => return Err(format!("Unexpected token in INPUTS section: {:?}", self.current_token()).to_string()),
            }
        }

        Ok(inputs)
    }
}

impl Parser {
    fn parse_outputs_section(&mut self) -> Result<Vec<String>, String> {
        self.expect(TokenKind::Outputs)?; // Expect "OUTPUTS"
        let mut inputs = vec![];

        loop {
            match self.current_token() {
                Some(TokenKind::Identifier(name)) => {
                    inputs.push(name.clone());
                    self.advance();
                }
                Some(TokenKind::Comma) => self.advance(),
                Some(TokenKind::Newline) => {
                    self.advance();
                    break;
                }
                _ => return Err(format!("Unexpected token in OUTPUTS section: {:?}", self.current_token()).to_string()),
            }
        }

        Ok(inputs)
    }
}

impl Parser {
    fn parse_component_list(&mut self) -> Result<Vec<Component>, String> {
        let mut components = vec![];

        while let Some(token) = self.current_token() {
            match token {
                TokenKind::And | TokenKind::Or | TokenKind::Not 
                | TokenKind::Nand | TokenKind::Nor | TokenKind::Xor | TokenKind::Xnor | TokenKind::Identifier(_) => {
                    components.push(self.parse_component()?);
                }
                TokenKind::Newline => {
                    self.advance();
                }
                TokenKind::End | TokenKind::EOF => break,
                _ => return Err(format!("Unexpected token in component list: {:?}", self.current_token()).to_string()),
            }
        }

        Ok(components)
    }

    fn parse_component(&mut self) -> Result<Component, String> {

        let gate_type = match self.current_token() {
            Some(TokenKind::And) => GateType::And,
            Some(TokenKind::Or) => GateType::Or,
            Some(TokenKind::Not) => GateType::Not,
            Some(TokenKind::Nand) => GateType::Nand,
            Some(TokenKind::Nor) => GateType::Nor,
            Some(TokenKind::Xor) => GateType::Xor,
            Some(TokenKind::Xnor) => GateType::Xnor,
            Some(TokenKind::Identifier(name)) => {
                GateType::Subcircuit(name.clone())
            }
            _ => return Err(format!(
                "Unexpected token for gate type: {:?}",
                self.current_token()
            )),
        };
    
        self.advance();
    
        let identifier = if let GateType::Subcircuit(_) = gate_type {
            "".to_string()
        } else {
            if let Some(TokenKind::Identifier(name)) = self.current_token() {
                let id = name.clone();
                self.advance();
                id
            } else {
                return Err("Expected identifier for component".to_string());
            }
        };
    
        self.expect(TokenKind::In)?;
        self.expect(TokenKind::ParenOpen)?;
    
        let mut inputs = vec![];
        while let Some(TokenKind::Identifier(name)) = self.current_token() {
            inputs.push(name.clone());
            self.advance();
            if let Some(TokenKind::Comma) = self.current_token() {
                self.advance();
            } else {
                break;
            }
        }
    
        self.expect(TokenKind::ParenClose)?;

        
        self.expect(TokenKind::Out)?;
        self.expect(TokenKind::ParenOpen)?;
    
        let mut outputs = vec![];
        while let Some(TokenKind::Identifier(name)) = self.current_token() {
            outputs.push(name.clone());
            self.advance();
            if let Some(TokenKind::Comma) = self.current_token() {
                self.advance();
            } else {
                break; 
            }
        }
    
        self.expect(TokenKind::ParenClose)?;
        
        Ok(Component {
            gate_type,
            identifier,
            inputs,
            outputs,
        })
    }    
}
