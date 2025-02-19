#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_debug_implementations)]

//! <div align="center">
//! 
//!   <img src="https://raw.githubusercontent.com/SeaQL/sea-query/master/docs/SeaQL logo dual.png" width="320"/>
//! 
//!   <h1>SeaQuery</h1>
//! 
//!   <p>
//!     <strong>A database agnostic runtime query builder for Rust</strong>
//!   </p>
//! 
//!   [![crate](https://img.shields.io/crates/v/sea-query.svg)](https://crates.io/crates/sea-query)
//!   [![docs](https://docs.rs/sea-query/badge.svg)](https://docs.rs/sea-query)
//!   [![build status](https://github.com/SeaQL/sea-query/actions/workflows/rust.yml/badge.svg)](https://github.com/SeaQL/sea-query/actions/workflows/rust.yml)
//! 
//!   <sub>Built with ❤️ by 🌊🦀🐚</sub>
//! 
//! </div>
//! 
//! ## Introduction
//! 
//! SeaQuery is query builder to help you construct dynamic SQL queries in Rust.
//! You can construct expressions, queries and schema as abstract syntax trees using an ergonomic API.
//! We support MySQL, Postgres and SQLite behind a common interface that aligns their behaviour where appropriate.
//! 
//! This library is the foundation of upcoming projects: Document ORM (SeaORM) and Database Synchor (SeaHorse).
//! 
//! ## Install
//! 
//! ```toml
//! # Cargo.toml
//! [dependencies]
//! sea-query = "*"
//! ```
//! 
//! ## Usage
//! 
//! Table of Content
//! 
//! 1. Basics
//! 
//!     1. [Iden](#iden)
//!     1. [Expression](#expression)
//!     1. [Builder](#builder)
//! 
//! 1. Query Statement
//! 
//!     1. [Query Select](#query-select)
//!     1. [Query Insert](#query-insert)
//!     1. [Query Update](#query-update)
//!     1. [Query Delete](#query-delete)
//! 
//! 1. Table Statement
//! 
//!     1. [Table Create](#table-create)
//!     1. [Table Alter](#table-alter)
//!     1. [Table Drop](#table-drop)
//!     1. [Table Rename](#table-rename)
//!     1. [Table Truncate](#table-truncate)
//!     1. [Foreign Key Create](#foreign-key-create)
//!     1. [Foreign Key Drop](#foreign-key-drop)
//!     1. [Index Create](#index-create)
//!     1. [Index Drop](#index-drop)
//! 
//! ### Drivers
//! 
//! We provide integration for [SQLx](https://crates.io/crates/sqlx), 
//! [postgres](https://crates.io/crates/postgres) and [rusqlite](https://crates.io/crates/rusqlite).
//! See [examples](https://github.com/SeaQL/sea-query/blob/master/examples) for usage.
//! 
//! ### Iden
//! 
//! `Iden` is a trait for identifiers used in any query statement.
//! 
//! Commonly implemented by Enum where each Enum represent a table found in a database,
//! and its variants include table name and column name.
//! 
//! [`Iden::unquoted()`] must be implemented to provide a mapping between Enum variant and its corresponding string value.
//! 
//! ```rust
//! use sea_query::{*, tests_cfg::*};
//! 
//! // For example Character table with column id, character, font_size...
//! pub enum Character {
//!     Table,
//!     Id,
//!     Character,
//!     FontSize,
//!     SizeW,
//!     SizeH,
//!     FontId,
//! }
//! 
//! // Mapping between Enum variant and its corresponding string value
//! impl Iden for Character {
//!     fn unquoted(&self, s: &mut dyn FmtWrite) {
//!         write!(s, "{}", match self {
//!             Self::Table => "character",
//!             Self::Id => "id",
//!             Self::Character => "character",
//!             Self::FontSize => "font_size",
//!             Self::SizeW => "size_w",
//!             Self::SizeH => "size_h",
//!             Self::FontId => "font_id",
//!         }).unwrap();
//!     }
//! }
//! ```
//! 
//! If you're okay with running another procedural macro, you can activate
//! the `derive` feature on the crate to save you some boilerplate.
//! For more information, look at 
//! [the derive example](https://github.com/SeaQL/sea-query/blob/master/examples/derive.rs).
//! 
//! ```rust
//! # #[cfg(feature = "derive")]
//! use sea_query::Iden;
//! 
//! // This will implement Iden exactly as shown above
//! # #[cfg(feature = "derive")]
//! #[derive(Iden)]
//! pub enum Character {
//!     Table,
//!     Id,
//!     Character,
//!     FontSize,
//!     SizeW,
//!     SizeH,
//!     FontId,
//! }
//! ```
//! 
//! ### Expression
//! 
//! Use [`Expr`] to construct select, join, where and having expression in query.
//! 
//! ```rust
//! use sea_query::{*, tests_cfg::*};
//! 
//! assert_eq!(
//!     Query::select()
//!         .column(Char::Character)
//!         .from(Char::Table)
//!         .left_join(Font::Table, Expr::tbl(Char::Table, Char::FontId).equals(Font::Table, Font::Id))
//!         .and_where(
//!             Expr::expr(Expr::col(Char::SizeW).add(1)).mul(2)
//!                 .equals(Expr::expr(Expr::col(Char::SizeH).div(2)).sub(1))
//!         )
//!         .and_where(Expr::col(Char::SizeW).in_subquery(
//!             Query::select()
//!                 .expr(Expr::cust("3 + 2 * 2"))
//!                 .take()
//!         ))
//!         .or_where(Expr::col(Char::Character).like("D").and(Expr::col(Char::Character).like("E")))
//!         .to_string(MysqlQueryBuilder),
//!     vec![
//!         "SELECT `character` FROM `character`",
//!         "LEFT JOIN `font` ON `character`.`font_id` = `font`.`id`",
//!         "WHERE ((`size_w` + 1) * 2 = (`size_h` / 2) - 1)",
//!             "AND `size_w` IN (SELECT 3 + 2 * 2)",
//!             "OR ((`character` LIKE 'D') AND (`character` LIKE 'E'))",
//!     ].join(" ")
//! );
//! ```
//! 
//! ### Builder
//! 
//! All the query statements and table statements support the following ways to build database specific SQL statement:
//! 
//! 1. `build(&self, query_builder: T) -> (String, Values)`  
//!     Build a SQL statement as string and parameters as a vector of values, see [here](https://docs.rs/sea-query/*/sea_query/query/struct.SelectStatement.html#method.build) for example.
//! 
//! 1. `build_collect(&self, query_builder: T, collector: &mut dyn FnMut(Value)) -> String`  
//!     Build a SQL statement as string and collect paramaters (usually for binding to binary protocol), see [here](https://docs.rs/sea-query/*/sea_query/query/struct.SelectStatement.html#method.build_collect) for example.
//! 
//! 1. `to_string(&self, query_builder: T) -> String`  
//!     Build a SQL statement as string with parameters injected, see [here](https://docs.rs/sea-query/*/sea_query/query/struct.SelectStatement.html#method.to_string) for example.
//! 
//! ### Query Select
//! 
//! ```rust
//! use sea_query::{*, tests_cfg::*};
//! 
//! let query = Query::select()
//!     .column(Char::Character)
//!     .column((Font::Table, Font::Name))
//!     .from(Char::Table)
//!     .left_join(Font::Table, Expr::tbl(Char::Table, Char::FontId).equals(Font::Table, Font::Id))
//!     .and_where(Expr::col(Char::SizeW).is_in(vec![3, 4]))
//!     .and_where(Expr::col(Char::Character).like("A%"))
//!     .to_owned();
//! 
//! assert_eq!(
//!     query.to_string(MysqlQueryBuilder),
//!     r#"SELECT `character`, `font`.`name` FROM `character` LEFT JOIN `font` ON `character`.`font_id` = `font`.`id` WHERE `size_w` IN (3, 4) AND `character` LIKE 'A%'"#
//! );
//! assert_eq!(
//!     query.to_string(PostgresQueryBuilder),
//!     r#"SELECT "character", "font"."name" FROM "character" LEFT JOIN "font" ON "character"."font_id" = "font"."id" WHERE "size_w" IN (3, 4) AND "character" LIKE 'A%'"#
//! );
//! assert_eq!(
//!     query.to_string(SqliteQueryBuilder),
//!     r#"SELECT `character`, `font`.`name` FROM `character` LEFT JOIN `font` ON `character`.`font_id` = `font`.`id` WHERE `size_w` IN (3, 4) AND `character` LIKE 'A%'"#
//! );
//! ```
//! 
//! ### Query Insert
//! 
//! ```rust
//! use sea_query::{*, tests_cfg::*};
//! 
//! let query = Query::insert()
//!     .into_table(Glyph::Table)
//!     .columns(vec![
//!         Glyph::Aspect,
//!         Glyph::Image,
//!     ])
//!     .values_panic(vec![
//!         5.15.into(),
//!         "12A".into(),
//!     ])
//!     .values_panic(vec![
//!         4.21.into(),
//!         "123".into(),
//!     ])
//!     .to_owned();
//! 
//! assert_eq!(
//!     query.to_string(MysqlQueryBuilder),
//!     r#"INSERT INTO `glyph` (`aspect`, `image`) VALUES (5.15, '12A'), (4.21, '123')"#
//! );
//! assert_eq!(
//!     query.to_string(PostgresQueryBuilder),
//!     r#"INSERT INTO "glyph" ("aspect", "image") VALUES (5.15, '12A'), (4.21, '123')"#
//! );
//! assert_eq!(
//!     query.to_string(SqliteQueryBuilder),
//!     r#"INSERT INTO `glyph` (`aspect`, `image`) VALUES (5.15, '12A'), (4.21, '123')"#
//! );
//! ```
//! 
//! ### Query Update
//! 
//! ```rust
//! use sea_query::{*, tests_cfg::*};
//! 
//! let query = Query::update()
//!     .table(Glyph::Table)
//!     .values(vec![
//!         (Glyph::Aspect, 1.23.into()),
//!         (Glyph::Image, "123".into()),
//!     ])
//!     .and_where(Expr::col(Glyph::Id).eq(1))
//!     .to_owned();
//! 
//! assert_eq!(
//!     query.to_string(MysqlQueryBuilder),
//!     r#"UPDATE `glyph` SET `aspect` = 1.23, `image` = '123' WHERE `id` = 1"#
//! );
//! assert_eq!(
//!     query.to_string(PostgresQueryBuilder),
//!     r#"UPDATE "glyph" SET "aspect" = 1.23, "image" = '123' WHERE "id" = 1"#
//! );
//! assert_eq!(
//!     query.to_string(SqliteQueryBuilder),
//!     r#"UPDATE `glyph` SET `aspect` = 1.23, `image` = '123' WHERE `id` = 1"#
//! );
//! ```
//! 
//! ### Query Delete
//! 
//! ```rust
//! use sea_query::{*, tests_cfg::*};
//! 
//! let query = Query::delete()
//!     .from_table(Glyph::Table)
//!     .or_where(Expr::col(Glyph::Id).lt(1))
//!     .or_where(Expr::col(Glyph::Id).gt(10))
//!     .to_owned();
//! 
//! assert_eq!(
//!     query.to_string(MysqlQueryBuilder),
//!     r#"DELETE FROM `glyph` WHERE (`id` < 1) OR (`id` > 10)"#
//! );
//! assert_eq!(
//!     query.to_string(PostgresQueryBuilder),
//!     r#"DELETE FROM "glyph" WHERE ("id" < 1) OR ("id" > 10)"#
//! );
//! assert_eq!(
//!     query.to_string(SqliteQueryBuilder),
//!     r#"DELETE FROM `glyph` WHERE (`id` < 1) OR (`id` > 10)"#
//! );
//! ```
//! 
//! ### Table Create
//! 
//! ```rust
//! use sea_query::{*, tests_cfg::*};
//! 
//! let table = Table::create()
//!     .table(Char::Table)
//!     .if_not_exists()
//!     .col(ColumnDef::new(Char::Id).integer().not_null().auto_increment().primary_key())
//!     .col(ColumnDef::new(Char::FontSize).integer().not_null())
//!     .col(ColumnDef::new(Char::Character).string().not_null())
//!     .col(ColumnDef::new(Char::SizeW).integer().not_null())
//!     .col(ColumnDef::new(Char::SizeH).integer().not_null())
//!     .col(ColumnDef::new(Char::FontId).integer().default(Value::Null))
//!     .foreign_key(
//!         ForeignKey::create()
//!             .name("FK_2e303c3a712662f1fc2a4d0aad6")
//!             .from(Char::Table, Char::FontId)
//!             .to(Font::Table, Font::Id)
//!             .on_delete(ForeignKeyAction::Cascade)
//!             .on_update(ForeignKeyAction::Cascade)
//!     )
//!     .to_owned();
//! 
//! assert_eq!(
//!     table.to_string(MysqlQueryBuilder),
//!     vec![
//!         r#"CREATE TABLE IF NOT EXISTS `character` ("#,
//!             r#"`id` int NOT NULL AUTO_INCREMENT PRIMARY KEY,"#,
//!             r#"`font_size` int NOT NULL,"#,
//!             r#"`character` varchar(255) NOT NULL,"#,
//!             r#"`size_w` int NOT NULL,"#,
//!             r#"`size_h` int NOT NULL,"#,
//!             r#"`font_id` int DEFAULT NULL,"#,
//!             r#"CONSTRAINT `FK_2e303c3a712662f1fc2a4d0aad6`"#,
//!                 r#"FOREIGN KEY (`font_id`) REFERENCES `font` (`id`)"#,
//!                 r#"ON DELETE CASCADE ON UPDATE CASCADE"#,
//!         r#")"#,
//!     ].join(" ")
//! );
//! assert_eq!(
//!     table.to_string(PostgresQueryBuilder),
//!     vec![
//!         r#"CREATE TABLE IF NOT EXISTS "character" ("#,
//!             r#""id" serial NOT NULL PRIMARY KEY,"#,
//!             r#""font_size" integer NOT NULL,"#,
//!             r#""character" varchar NOT NULL,"#,
//!             r#""size_w" integer NOT NULL,"#,
//!             r#""size_h" integer NOT NULL,"#,
//!             r#""font_id" integer DEFAULT NULL,"#,
//!             r#"CONSTRAINT "FK_2e303c3a712662f1fc2a4d0aad6""#,
//!                 r#"FOREIGN KEY ("font_id") REFERENCES "font" ("id")"#,
//!                 r#"ON DELETE CASCADE ON UPDATE CASCADE"#,
//!         r#")"#,
//!     ].join(" ")
//! );
//! assert_eq!(
//!     table.to_string(SqliteQueryBuilder),
//!     vec![
//!        r#"CREATE TABLE IF NOT EXISTS `character` ("#,
//!            r#"`id` integer NOT NULL PRIMARY KEY AUTOINCREMENT,"#,
//!            r#"`font_size` integer NOT NULL,"#,
//!            r#"`character` text NOT NULL,"#,
//!            r#"`size_w` integer NOT NULL,"#,
//!            r#"`size_h` integer NOT NULL,"#,
//!            r#"`font_id` integer DEFAULT NULL,"#,
//!            r#"FOREIGN KEY (`font_id`) REFERENCES `font` (`id`) ON DELETE CASCADE ON UPDATE CASCADE"#,
//!        r#")"#,
//!     ].join(" ")
//! );
//! ```
//! 
//! ### Table Alter
//! 
//! ```rust
//! use sea_query::{*, tests_cfg::*};
//! 
//! let table = Table::alter()
//!     .table(Font::Table)
//!     .add_column(ColumnDef::new(Alias::new("new_col")).integer().not_null().default(100))
//!     .to_owned();
//! 
//! assert_eq!(
//!     table.to_string(MysqlQueryBuilder),
//!     r#"ALTER TABLE `font` ADD COLUMN `new_col` int NOT NULL DEFAULT 100"#
//! );
//! assert_eq!(
//!     table.to_string(PostgresQueryBuilder),
//!     r#"ALTER TABLE "font" ADD COLUMN "new_col" integer NOT NULL DEFAULT 100"#
//! );
//! assert_eq!(
//!     table.to_string(SqliteQueryBuilder),
//!     r#"ALTER TABLE `font` ADD COLUMN `new_col` integer NOT NULL DEFAULT 100"#,
//! );
//! ```
//! 
//! ### Table Drop
//! 
//! ```rust
//! use sea_query::{*, tests_cfg::*};
//! 
//! let table = Table::drop()
//!     .table(Glyph::Table)
//!     .table(Char::Table)
//!     .to_owned();
//! 
//! assert_eq!(
//!     table.to_string(MysqlQueryBuilder),
//!     r#"DROP TABLE `glyph`, `character`"#
//! );
//! assert_eq!(
//!     table.to_string(PostgresQueryBuilder),
//!     r#"DROP TABLE "glyph", "character""#
//! );
//! assert_eq!(
//!     table.to_string(SqliteQueryBuilder),
//!     r#"DROP TABLE `glyph`, `character`"#
//! );
//! ```
//! 
//! ### Table Rename
//! 
//! ```rust
//! use sea_query::{*, tests_cfg::*};
//! 
//! let table = Table::rename()
//!     .table(Font::Table, Alias::new("font_new"))
//!     .to_owned();
//! 
//! assert_eq!(
//!     table.to_string(MysqlQueryBuilder),
//!     r#"RENAME TABLE `font` TO `font_new`"#
//! );
//! assert_eq!(
//!     table.to_string(PostgresQueryBuilder),
//!     r#"ALTER TABLE "font" RENAME TO "font_new""#
//! );
//! assert_eq!(
//!     table.to_string(SqliteQueryBuilder),
//!     r#"ALTER TABLE `font` RENAME TO `font_new`"#
//! );
//! ```
//! 
//! ### Table Truncate
//! 
//! ```rust
//! use sea_query::{*, tests_cfg::*};
//! 
//! let table = Table::truncate()
//!     .table(Font::Table)
//!     .to_owned();
//! 
//! assert_eq!(
//!     table.to_string(MysqlQueryBuilder),
//!     r#"TRUNCATE TABLE `font`"#
//! );
//! assert_eq!(
//!     table.to_string(PostgresQueryBuilder),
//!     r#"TRUNCATE TABLE "font""#
//! );
//! assert_eq!(
//!     table.to_string(SqliteQueryBuilder),
//!     r#"TRUNCATE TABLE `font`"#
//! );
//! ```
//! 
//! ### Foreign Key Create
//! 
//! ```rust
//! use sea_query::{*, tests_cfg::*};
//! 
//! let foreign_key = ForeignKey::create()
//!     .name("FK_character_font")
//!     .from(Char::Table, Char::FontId)
//!     .to(Font::Table, Font::Id)
//!     .on_delete(ForeignKeyAction::Cascade)
//!     .on_update(ForeignKeyAction::Cascade)
//!     .to_owned();
//! 
//! assert_eq!(
//!     foreign_key.to_string(MysqlQueryBuilder),
//!     vec![
//!         r#"ALTER TABLE `character`"#,
//!         r#"ADD CONSTRAINT `FK_character_font`"#,
//!         r#"FOREIGN KEY (`font_id`) REFERENCES `font` (`id`)"#,
//!         r#"ON DELETE CASCADE ON UPDATE CASCADE"#,
//!     ].join(" ")
//! );
//! assert_eq!(
//!     foreign_key.to_string(PostgresQueryBuilder),
//!     vec![
//!         r#"ALTER TABLE "character" ADD CONSTRAINT "FK_character_font""#,
//!         r#"FOREIGN KEY ("font_id") REFERENCES "font" ("id")"#,
//!         r#"ON DELETE CASCADE ON UPDATE CASCADE"#,
//!     ].join(" ")
//! );
//! // Sqlite does not support modification of foreign key constraints to existing tables
//! ```
//! 
//! ### Foreign Key Drop
//! 
//! ```rust
//! use sea_query::{*, tests_cfg::*};
//! 
//! let foreign_key = ForeignKey::drop()
//!     .name("FK_character_font")
//!     .table(Char::Table)
//!     .to_owned();
//! 
//! assert_eq!(
//!     foreign_key.to_string(MysqlQueryBuilder),
//!     r#"ALTER TABLE `character` DROP FOREIGN KEY `FK_character_font`"#
//! );
//! assert_eq!(
//!     foreign_key.to_string(PostgresQueryBuilder),
//!     r#"ALTER TABLE "character" DROP CONSTRAINT "FK_character_font""#
//! );
//! // Sqlite does not support modification of foreign key constraints to existing tables
//! ```
//! 
//! ### Index Create
//! 
//! ```rust
//! use sea_query::{*, tests_cfg::*};
//! 
//! let index = Index::create()
//!     .name("idx-glyph-aspect")
//!     .table(Glyph::Table)
//!     .col(Glyph::Aspect)
//!     .to_owned();
//! 
//! assert_eq!(
//!     index.to_string(MysqlQueryBuilder),
//!     r#"CREATE INDEX `idx-glyph-aspect` ON `glyph` (`aspect`)"#
//! );
//! assert_eq!(
//!     index.to_string(PostgresQueryBuilder),
//!     r#"CREATE INDEX "idx-glyph-aspect" ON "glyph" ("aspect")"#
//! );
//! assert_eq!(
//!     index.to_string(SqliteQueryBuilder),
//!     r#"CREATE INDEX `idx-glyph-aspect` ON `glyph` (`aspect`)"#
//! );
//! ```
//! 
//! ### Index Drop
//! 
//! ```rust
//! use sea_query::{*, tests_cfg::*};
//! 
//! let index = Index::drop()
//!     .name("idx-glyph-aspect")
//!     .table(Glyph::Table)
//!     .to_owned();
//! 
//! assert_eq!(
//!     index.to_string(MysqlQueryBuilder),
//!     r#"DROP INDEX `idx-glyph-aspect` ON `glyph`"#
//! );
//! assert_eq!(
//!     index.to_string(PostgresQueryBuilder),
//!     r#"DROP INDEX "idx-glyph-aspect""#
//! );
//! assert_eq!(
//!     index.to_string(SqliteQueryBuilder),
//!     r#"DROP INDEX `idx-glyph-aspect` ON `glyph`"#
//! );
//! ```

pub mod backend;
pub mod driver;
pub mod extension;
pub mod foreign_key;
pub mod index;
pub mod query;
pub mod table;
pub mod error;
pub mod expr;
pub mod func;
pub mod prepare;
pub mod schema;
pub mod tests_cfg;
pub mod token;
pub mod types;
pub mod value;

pub use backend::*;
pub use driver::*;
//pub use extension::*;
pub use foreign_key::*;
pub use index::*;
pub use query::*;
pub use table::*;
//pub use error::*;
pub use expr::*;
pub use func::*;
pub use prepare::*;
pub use schema::*;
//pub use tests_cfg::*;
pub use token::*;
pub use types::*;
pub use value::*;

#[cfg(feature = "derive")]
pub use sea_query_derive::Iden;