//! Common types used in the library.

use std::fmt;
use std::rc::Rc;
use crate::{query::*, expr::*};

/// Identifier in query
pub trait Iden {
    fn prepare(&self, s: &mut dyn fmt::Write, q: char) {
        write!(s, "{}", q).unwrap();
        self.unquoted(s);
        write!(s, "{}", q).unwrap();
    }

    fn to_string(&self) -> String {
        let s = &mut String::new();
        self.unquoted(s);
        s.to_owned()
    }

    fn unquoted(&self, s: &mut dyn fmt::Write);
}

impl fmt::Debug for dyn Iden {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.unquoted(formatter);
        Ok(())
    }
}

/// Column references
#[derive(Debug, Clone)]
pub enum ColumnRef {
    Column(Rc<dyn Iden>),
    TableColumn(Rc<dyn Iden>, Rc<dyn Iden>),
}

/// Table references
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum TableRef {
    Table(Rc<dyn Iden>),
    SchemaTable(Rc<dyn Iden>, Rc<dyn Iden>),
    TableAlias(Rc<dyn Iden>, Rc<dyn Iden>),
    SchemaTableAlias(Rc<dyn Iden>, Rc<dyn Iden>, Rc<dyn Iden>),
    SubQuery(SelectStatement, Rc<dyn Iden>),
}

/// Unary operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOper {
    Not,
}

/// Binary operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOper {
    And,
    Or,
    Like,
    NotLike,
    Is,
    IsNot,
    In,
    NotIn,
    Between,
    NotBetween,
    Equal,
    NotEqual,
    SmallerThan,
    GreaterThan,
    SmallerThanOrEqual,
    GreaterThanOrEqual,
    Add,
    Sub,
    Mul,
    Div,
}

/// Logical chain operator
#[derive(Debug, Clone)]
pub enum LogicalChainOper {
    And(SimpleExpr),
    Or(SimpleExpr),
}

/// Join types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    Join,
    InnerJoin,
    LeftJoin,
    RightJoin,
}

/// Order expression
#[derive(Debug, Clone)]
pub struct OrderExpr {
    pub(crate) expr: SimpleExpr,
    pub(crate) order: Order,
}

/// Join on types
#[derive(Debug, Clone)]
pub enum JoinOn {
    Condition(Box<SimpleExpr>),
    Columns(Vec<SimpleExpr>),
}

/// Ordering options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    Asc,
    Desc,
}

/// Helper for create name alias
#[derive(Debug, Clone)]
pub struct Alias(String);

/// Common SQL Keywords
#[derive(Debug, Clone)]
pub enum Keyword {
    Null,
    Custom(Rc<dyn Iden>),
}

// Impl begins

impl Into<ColumnRef> for dyn Iden + 'static {
    fn into(self) -> ColumnRef {
        ColumnRef::Column(self.into())
    }
}

impl Into<ColumnRef> for (dyn Iden + 'static, dyn Iden + 'static) {
    fn into(self) -> ColumnRef {
        ColumnRef::TableColumn(self.0.into(), self.1.into())
    }
}

impl Into<TableRef> for dyn Iden + 'static {
    fn into(self) -> TableRef {
        TableRef::Table(self.into())
    }
}

impl Into<TableRef> for (dyn Iden + 'static, dyn Iden + 'static) {
    fn into(self) -> TableRef {
        TableRef::SchemaTable(self.0.into(), self.1.into())
    }
}

impl TableRef {
    pub fn alias<A>(self, alias: A) -> Self
    where
        A: Into<Rc<dyn Iden + 'static>>,
    {
        match self {
            Self::Table(table) => Self::TableAlias(table, alias.into()),
            Self::SchemaTable(schema, table) => Self::SchemaTableAlias(schema, table, alias.into()),
            _ => panic!("unexpected TableRef variant")
        }
    }
}

impl Alias {
    pub fn new(n: &str) -> Self {
        Self(n.to_owned())
    }
}

impl Iden for Alias {
    fn unquoted(&self, s: &mut dyn fmt::Write) {
        write!(s, "{}", self.0).unwrap();
    }
}