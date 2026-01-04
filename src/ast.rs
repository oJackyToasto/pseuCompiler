#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(String, Span),
    String(String, Span),
    Char(String, Span),
    Variable(String, Span),
    Boolean(bool, Span),
    BinaryOp(Box<Expr>, BinaryOp, Box<Expr>, Span),   
    UnaryOp(UnaryOp, Box<Expr>, Span),

    FunctionCall {
        name: String,
        args: Vec<Expr>,
        span: Span,
    },

    ArrayAccess {
        array: String,
        indices: Vec<Expr>,
        span: Span,
    },

    FieldAccess {
        object: Box<Expr>,
        field: String,
        span: Span,
    },

    PointerDeref {
        pointer: Box<Expr>,
        span: Span,
    },

    PointerRef {
        target: Box<Expr>,
        span: Span,
    },

    // SetAccess {
    //     set: Box<Expr>,
    //     element: Box<Expr>,
    // },

    // EnumAccess {
    //     enum_type: String,
    //     value: String,
    // },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    _Div,  // Integer division
    Modulus,
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    And,
    Or,
}

impl BinaryOp {
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOp::Or => 1,
            BinaryOp::And => 2,
            BinaryOp::Equals | BinaryOp::NotEquals 
            | BinaryOp::LessThan | BinaryOp::GreaterThan
            | BinaryOp::LessThanOrEqual | BinaryOp::GreaterThanOrEqual => 3,
            BinaryOp::Add | BinaryOp::Subtract => 4,
            BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::_Div | BinaryOp::Modulus => 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Not,
    Negate,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileMode {
    READ,
    WRITE,
    RANDOM
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Procedure {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
    pub span: Span,
}   

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub type_name: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {

    TypeDeclaration {
        name: String,
        variant: TypeDeclarationVariant,
        span: Span,
    },

    Define {
        name: String,
        values: Vec<String>,
        type_name: String,
        span: Span,
    },

    Declare {
        name: String,
        type_name: Type,
        initial_value: Option<Box<Expr>>,
        span: Span,
    },

    Assign {
        name: String,
        indices: Option<Vec<Expr>>,
        expression: Box<Expr>,
        span: Span,
    },

    Constant {
        name: String,
        value: Option<Box<Expr>>,  // None means lock with current value
        span: Span,
    },

    If {
        condition: Box<Expr>,
        then_stmt: Vec<Stmt>,
        else_stmt: Option<Vec<Stmt>>,
        span: Span,
    },

    While {
        condition: Box<Expr>,
        body: Vec<Stmt>,
        span: Span,
    },

    For {
        counter: String,
        start: Box<Expr>,
        end: Box<Expr>,
        step: Option<Box<Expr>>,
        body: Vec<Stmt>,
        span: Span,
    },

    RepeatUntil {
        body: Vec<Stmt>,
        condition: Box<Expr>,
        span: Span,
    },

    OpenFile {
        filename: Box<Expr>,
        mode: FileMode,
        span: Span,
    },

    CloseFile {
        filename: Box<Expr>,
        span: Span,
    },

    WriteFile {
        filename: Box<Expr>,
        exprs: Vec<Expr>,
        span: Span,
    },

    ReadFile {
        filename: Box<Expr>,
        name: String,
        span: Span,
    },
    
    Seek {
        filename: Box<Expr>,
        address: Box<Expr>,
        span: Span,
    },

    GetRecord {
        filename: Box<Expr>,
        variable: String,
        span: Span,
    },
    
    PutRecord {
        filename: Box<Expr>,
        variable: String,
        span: Span,
    },

    Return {
        value: Option<Box<Expr>>,
        span: Span,
    },

    Call {
        name: String,
        args: Option<Vec<Expr>>,
        span: Span,
    },

    Input {
        name: String,
        span: Span,
    },
    
    Output {
        exprs: Vec<Expr>,
        span: Span,
    },

    FunctionDeclaration {
        function: Function,
        span: Span,
    },

    ProcedureDeclaration {
        procedure: Procedure,
        span: Span,
    },

    Case {
        expression: Box<Expr>,
        cases: Vec<CaseBranch>,
        otherwise: Option<Vec<Stmt>>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeField {
    pub name: String,
    pub type_name: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeDeclarationVariant {
    Record {
        fields: Vec<TypeField>,
    },
    Enum {
        values: Vec<String>,
    },
    Pointer {
        points_to: Box<Type>,
    },
    Set {
        element_type: Box<Type>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaseBranch {
    pub value: Box<Expr>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    INTEGER,
    REAL,
    STRING,
    CHAR,
    BOOLEAN,
    DATE,
    ARRAY {
        dimensions: Vec<(Box<Expr>, Box<Expr>)>,
        element_type: Box<Type>,
    },
    Custom(String),
    Enum {
        name: String,
        values: Vec<String>,
    },
    Record {
        name: String,
        fields: Vec<TypeField>,
    },
    Pointer {
        points_to: Box<Type>,
    },
    Set {
        element_type: Box<Type>,
    },
}