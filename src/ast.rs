#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(String),
    String(String),
    Char(String),
    Variable(String),
    BinaryOp(Box<Expr>, BinaryOp, Box<Expr>),   
    UnaryOp(UnaryOp, Box<Expr>),

    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },

    ArrayAccess {
        array: String,
        index: Box<Expr>,
    },

    FieldAccess {
        object: Box<Expr>,
        field: String,
    },

    PointerDeref {
        pointer: Box<Expr>,
    },

    PointerRef {
        target: Box<Expr>,
    },

    SetAccess {
        set: Box<Expr>,
        element: Box<Expr>,
    },

    EnumAccess {
        enum_type: String,
        value: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Div,  // Integer division
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
            BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Div | BinaryOp::Modulus => 5,
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
    name: String,
    params: Vec<Param>,
    return_type: Type,
    body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Procedure {
    name: String,
    params: Vec<Param>,
    body: Vec<Stmt>,
}   

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    name: String,
    type_name: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {

    TypeDeclaration {
        name: String,
        variant: TypeDeclarationVariant,
    },

    Define {
        name: String,
        values: Vec<String>,
        type_name: String,
    },

    Declare {
        name: String,
        type_name: Type,
        initial_value: Option<Box<Expr>>,
    },

    Assign {
        name: String,
        index: Option<Box<Expr>>,
        expression: Box<Expr>,
    },

    If {
        condition: Box<Expr>,
        then_stmt: Vec<Stmt>,
        else_stmt: Option<Vec<Stmt>>,
    },

    While {
        condition: Box<Expr>,
        body: Vec<Stmt>,
    },

    For {
        counter: String,
        start: Box<Expr>,
        end: Box<Expr>,
        step: Option<Box<Expr>>,
        body: Vec<Stmt>,
    },

    RepeatUntil {
        body: Vec<Stmt>,
        condition: Box<Expr>,
    },

    OpenFile {
        filename: Box<Expr>,
        mode: FileMode,
    },

    CloseFile {
        filename: Box<Expr>,
    },

    WriteFile {
        filename: Box<Expr>,
        exprs: Vec<Expr>,
    },

    ReadFile {
        filename: Box<Expr>,
        name: String,
    },
    
    Seek {
        filename: Box<Expr>,
        address: Box<Expr>,
    },

    GetRecord {
        filename: Box<Expr>,
        variable: String,
    },
    
    PutRecord {
        filename: Box<Expr>,
        variable: String,
    },

    Return {
        value: Option<Box<Expr>>,
    },

    Break,

    Call {
        name: String,
        args: Option<Vec<Expr>>,
    },

    Input {
        name: String,
    },
    
    Output {
        exprs: Vec<Expr>,
    },

    FunctionDeclaration {
        function: Function,
    },

    ProcedureDeclaration {
        procedure: Procedure,
    },

    Case {
        expression: Box<Expr>,
        cases: Vec<CaseBranch>,
        otherwise: Option<Vec<Stmt>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeField {
    name: String,
    type_name: Type,
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
    Pointer {
        points_to: Box<Type>,
    },
    Set {
        element_type: Box<Type>,
    },
}