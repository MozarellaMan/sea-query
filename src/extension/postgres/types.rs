use crate::{backend::QueryBuilder, prepare::*, types::*, value::*};
use std::rc::Rc;

/// Helper for constructing any type statement
#[derive(Debug)]
pub struct Type;

#[derive(Debug, Clone, Default)]
pub struct TypeCreateStatement {
    pub(crate) name: Option<Rc<dyn Iden>>,
    pub(crate) as_type: Option<TypeAs>,
    pub(crate) values: Vec<Rc<dyn Iden>>,
}

#[derive(Debug, Clone)]
pub enum TypeAs {
    // Composite,
    Enum,
    // Range,
    // Base,
    // Array,
}

#[derive(Debug, Clone, Default)]
pub struct TypeDropStatement {
    pub(crate) names: Vec<Rc<dyn Iden>>,
    pub(crate) option: Option<TypeDropOpt>,
    pub(crate) if_exists: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TypeAlterStatement {
    pub(crate) name: Option<Rc<dyn Iden>>,
    pub(crate) option: Option<TypeAlterOpt>,
}

#[derive(Debug, Clone)]
pub enum TypeDropOpt {
    Cascade,
    Restrict,
}

#[derive(Debug, Clone)]
pub enum TypeAlterOpt {
    Add(Rc<dyn Iden>, Option<TypeAlterAddOpt>),
    Rename(Rc<dyn Iden>),
    RenameValue(Rc<dyn Iden>, Rc<dyn Iden>)
}

#[derive(Debug, Clone)]
pub enum TypeAlterAddOpt {
    Before(Rc<dyn Iden>),
    After(Rc<dyn Iden>),
}

pub trait TypeBuilder {
    /// Translate [`TypeCreateStatement`] into database specific SQL statement.
    fn prepare_type_create_statement(
        &self,
        create: &TypeCreateStatement,
        sql: &mut SqlWriter,
        collector: &mut dyn FnMut(Value),
    );

    /// Translate [`TypeDropStatement`] into database specific SQL statement.
    fn prepare_type_drop_statement(
        &self,
        drop: &TypeDropStatement,
        sql: &mut SqlWriter,
        collector: &mut dyn FnMut(Value),
    );

    /// Translate [`TypeAlterStatement`] into database specific SQL statement.
    fn prepare_type_alter_statement(
        &self,
        alter: &TypeAlterStatement,
        sql: &mut SqlWriter,
        collector: &mut dyn FnMut(Value),
    );
}

impl Type {
    /// Construct type [`TypeCreateStatement`]
    pub fn create() -> TypeCreateStatement {
        TypeCreateStatement::new()
    }

    /// Construct type [`TypeDropStatement`]
    pub fn drop() -> TypeDropStatement {
        TypeDropStatement::new()
    }

    /// Construct type [`TypeAlterStatement`]
    pub fn alter() -> TypeAlterStatement {
        TypeAlterStatement::new()
    }
}

impl TypeCreateStatement {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create enum as custom type
    ///
    /// ```
    /// use sea_query::{*, extension::postgres::Type};
    ///
    /// enum FontFamily {
    ///     Type,
    ///     Serif,
    ///     Sans,
    ///     Monospace,
    /// }
    ///
    /// impl Iden for FontFamily {
    ///     fn unquoted(&self, s: &mut dyn std::fmt::Write) {
    ///         write!(s, "{}", match self {
    ///             Self::Type => "font_family",
    ///             Self::Serif => "serif",
    ///             Self::Sans => "sans",
    ///             Self::Monospace => "monospace",
    ///         }).unwrap();
    ///     }
    /// }
    ///
    /// assert_eq!(
    ///     Type::create()
    ///         .as_enum(FontFamily::Type)
    ///         .values(vec![FontFamily::Serif, FontFamily::Sans, FontFamily::Monospace])
    ///         .to_string(PostgresQueryBuilder),
    ///     r#"CREATE TYPE "font_family" AS ENUM ('serif', 'sans', 'monospace')"#
    /// );
    /// ```
    pub fn as_enum<T: 'static>(&mut self, name: T) -> &mut Self
    where
        T: Iden,
    {
        self.name = Some(Rc::new(name));
        self.as_type = Some(TypeAs::Enum);
        self
    }

    pub fn values<T, I>(&mut self, values: I) -> &mut Self
    where
        T: IntoIden,
        I: IntoIterator<Item = T>,
    {
        for v in values.into_iter() {
            self.values.push(v.into_iden());
        }
        self
    }

    // below are boiler plates

    pub fn build<T: TypeBuilder>(&self, type_builder: T) -> (String, Vec<Value>) {
        self.build_ref(&type_builder)
    }

    pub fn build_ref<T: TypeBuilder>(&self, type_builder: &T) -> (String, Vec<Value>) {
        let mut params = Vec::new();
        let mut collector = |v| params.push(v);
        let sql = self.build_collect_ref(type_builder, &mut collector);
        (sql, params)
    }

    pub fn build_collect<T: TypeBuilder>(
        &self,
        type_builder: T,
        collector: &mut dyn FnMut(Value),
    ) -> String {
        self.build_collect_ref(&type_builder, collector)
    }

    pub fn build_collect_ref<T: TypeBuilder>(
        &self,
        type_builder: &T,
        collector: &mut dyn FnMut(Value),
    ) -> String {
        let mut sql = SqlWriter::new();
        type_builder.prepare_type_create_statement(self, &mut sql, collector);
        sql.result()
    }

    /// Build corresponding SQL statement and return SQL string
    pub fn to_string<T>(&self, type_builder: T) -> String
    where
        T: TypeBuilder + QueryBuilder,
    {
        let (sql, values) = self.build_ref(&type_builder);
        inject_parameters(&sql, values, &type_builder)
    }
}

impl TypeDropStatement {
    pub fn new() -> Self {
        Self::default()
    }

    /// Drop a type
    ///
    /// ```
    /// use sea_query::{*, extension::postgres::Type};
    ///
    /// struct FontFamily;
    ///
    /// impl Iden for FontFamily {
    ///     fn unquoted(&self, s: &mut dyn std::fmt::Write) {
    ///         write!(s, "{}", "font_family").unwrap();
    ///     }
    /// }
    ///
    /// assert_eq!(
    ///     Type::drop()
    ///         .if_exists()
    ///         .name(FontFamily)
    ///         .restrict()
    ///         .to_string(PostgresQueryBuilder),
    ///     r#"DROP TYPE IF EXISTS "font_family" RESTRICT"#
    /// );
    /// ```
    pub fn name<T>(&mut self, name: T) -> &mut Self
    where
        T: IntoIden,
    {
        self.names.push(name.into_iden());
        self
    }

    pub fn names<T, I>(&mut self, names: I) -> &mut Self
    where
        T: IntoIden,
        I: IntoIterator<Item = T>,
    {
        for n in names.into_iter() {
            self.names.push(n.into_iden());
        }
        self
    }

    /// Set `IF EXISTS`
    pub fn if_exists(&mut self) -> &mut Self {
        self.if_exists = true;
        self
    }

    /// Set `CASCADE`
    pub fn cascade(&mut self) -> &mut Self {
        self.option = Some(TypeDropOpt::Cascade);
        self
    }

    /// Set `RESTRICT`
    pub fn restrict(&mut self) -> &mut Self {
        self.option = Some(TypeDropOpt::Restrict);
        self
    }

    // below are boiler plates

    pub fn build<T: TypeBuilder>(&self, type_builder: T) -> (String, Vec<Value>) {
        self.build_ref(&type_builder)
    }

    pub fn build_ref<T: TypeBuilder>(&self, type_builder: &T) -> (String, Vec<Value>) {
        let mut params = Vec::new();
        let mut collector = |v| params.push(v);
        let sql = self.build_collect_ref(type_builder, &mut collector);
        (sql, params)
    }

    pub fn build_collect<T: TypeBuilder>(
        &self,
        type_builder: T,
        collector: &mut dyn FnMut(Value),
    ) -> String {
        self.build_collect_ref(&type_builder, collector)
    }

    pub fn build_collect_ref<T: TypeBuilder>(
        &self,
        type_builder: &T,
        collector: &mut dyn FnMut(Value),
    ) -> String {
        let mut sql = SqlWriter::new();
        type_builder.prepare_type_drop_statement(self, &mut sql, collector);
        sql.result()
    }

    /// Build corresponding SQL statement and return SQL string
    pub fn to_string<T>(&self, type_builder: T) -> String
    where
        T: TypeBuilder + QueryBuilder,
    {
        let (sql, values) = self.build_ref(&type_builder);
        inject_parameters(&sql, values, &type_builder)
    }
}

impl TypeAlterStatement {
    pub fn new() -> Self {
        Self::default()
    }

    /// Change the definition of a type 
    ///
    /// ```
    /// use sea_query::{*, extension::postgres::Type};
    ///
    /// enum FontFamily {
    ///     Type,
    ///     Serif,
    ///     Sans,
    ///     Monospace,
    /// }
    ///
    /// impl Iden for FontFamily {
    ///     fn unquoted(&self, s: &mut dyn std::fmt::Write) {
    ///         write!(s, "{}", match self {
    ///             Self::Type => "font_family",
    ///             Self::Serif => "serif",
    ///             Self::Sans => "sans",
    ///             Self::Monospace => "monospace",
    ///         }).unwrap();
    ///     }
    /// }
    ///
    /// assert_eq!(
    ///     Type::alter()
    ///         .name(FontFamily::Type)
    ///         .add_value(Alias::new("cursive"))
    ///         .to_string(PostgresQueryBuilder),
    ///     r#"ALTER TYPE "font_family" ADD VALUE 'cursive'"#
    /// );
    /// ```
    pub fn name<T>(mut self, name: T) -> Self
    where
        T: IntoIden,
    {
        self.name = Some(name.into_iden());
        self
    }

    pub fn add_value<T>(self, value: T) -> Self
    where
        T: IntoIden,
    {
        self.alter_option(TypeAlterOpt::Add(value.into_iden(), None))
    }

    pub fn before<T>(mut self, value: T) -> Self
    where
        T: IntoIden,
    {
        if let Some(option) = self.option {
            self.option = Some(option.before(value));
        }
        self
    }

    pub fn after<T>(mut self, value: T) -> Self
    where
        T: IntoIden,
    {
        if let Some(option) = self.option {
            self.option = Some(option.after(value));
        }
        self
    }

    pub fn rename_to<T>(self, name: T) -> Self
    where
        T: IntoIden,
    {
        self.alter_option(TypeAlterOpt::Rename(name.into_iden()))
    }

    pub fn rename_value<T>(self, existing: T, new_name: T) -> Self 
    where
        T: IntoIden,
    {
        self.alter_option(TypeAlterOpt::RenameValue(existing.into_iden(), new_name.into_iden()))
    }

    fn alter_option(mut self, option: TypeAlterOpt) -> Self {
        self.option = Some(option);
        self
    }

    // below are boilerplate

    pub fn build<T: TypeBuilder>(&self, type_builder: T) -> (String, Vec<Value>) {
        self.build_ref(&type_builder)
    }

    pub fn build_ref<T: TypeBuilder>(&self, type_builder: &T) -> (String, Vec<Value>) {
        let mut params = Vec::new();
        let mut collector = |v| params.push(v);
        let sql = self.build_collect_ref(type_builder, &mut collector);
        (sql, params)
    }

    pub fn build_collect<T: TypeBuilder>(
        &self,
        type_builder: T,
        collector: &mut dyn FnMut(Value),
    ) -> String {
        self.build_collect_ref(&type_builder, collector)
    }

    pub fn build_collect_ref<T: TypeBuilder>(
        &self,
        type_builder: &T,
        collector: &mut dyn FnMut(Value),
    ) -> String {
        let mut sql = SqlWriter::new();
        type_builder.prepare_type_alter_statement(self, &mut sql, collector);
        sql.result()
    }

    /// Build corresponding SQL statement and return SQL string
    pub fn to_string<T>(&self, type_builder: T) -> String
    where
        T: TypeBuilder + QueryBuilder,
    {
        let (sql, values) = self.build_ref(&type_builder);
        inject_parameters(&sql, values, &type_builder)
    }
}

impl TypeAlterOpt {
    /// Changes only `ADD VALUE x` options into `ADD VALUE x BEFORE` options, does nothing otherwise
    pub fn before<T>(self, value: T) -> Self
    where
        T: IntoIden,
    {
        match self {
            TypeAlterOpt::Add(iden, _) => {
                Self::Add(iden, Some(TypeAlterAddOpt::Before(value.into_iden())))
            }
            _ => self,
        }
    }

    /// Changes only `ADD VALUE x` options into `ADD VALUE x AFTER` options, does nothing otherwise
    pub fn after<T>(self, value: T) -> Self
    where
        T: IntoIden,
    {
        match self {
            TypeAlterOpt::Add(iden, _) => {
                Self::Add(iden, Some(TypeAlterAddOpt::After(value.into_iden())))
            }
            _ => self,
        }
    }
}
