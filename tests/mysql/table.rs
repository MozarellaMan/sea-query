use super::*;

#[test]
fn create_1() {
    assert_eq!(
        Table::create()
            .table(Glyph::Table)
            .col(ColumnDef::new(Glyph::Id).integer_len(11).not_null().auto_increment().primary_key())
            .col(ColumnDef::new(Glyph::Aspect).double().not_null())
            .col(ColumnDef::new(Glyph::Image).text())
            .engine("InnoDB")
            .character_set("utf8mb4")
            .collate("utf8mb4_unicode_ci")
            .to_string(MysqlQueryBuilder),
        vec![
            "CREATE TABLE `glyph` (",
                "`id` int(11) NOT NULL AUTO_INCREMENT PRIMARY KEY,",
                "`aspect` double NOT NULL,",
                "`image` text",
            ") ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci",
        ].join(" ")
    );
}

#[test]
fn create_2() {
    assert_eq!(
        Table::create()
            .table(Font::Table)
            .col(ColumnDef::new(Font::Id).integer_len(11).not_null().auto_increment().primary_key())
            .col(ColumnDef::new(Font::Name).string().not_null())
            .col(ColumnDef::new(Font::Variant).string_len(255).not_null())
            .col(ColumnDef::new(Font::Language).string_len(1024).not_null())
            .engine("InnoDB")
            .character_set("utf8mb4")
            .collate("utf8mb4_unicode_ci")
            .to_string(MysqlQueryBuilder),
        vec![
            "CREATE TABLE `font` (",
                "`id` int(11) NOT NULL AUTO_INCREMENT PRIMARY KEY,",
                "`name` varchar(255) NOT NULL,",
                "`variant` varchar(255) NOT NULL,",
                "`language` varchar(1024) NOT NULL",
            ") ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci",
        ].join(" ")
    );
}

#[test]
fn create_3() {
    assert_eq!(
        Table::create()
            .table(Char::Table)
            .if_not_exists()
            .col(ColumnDef::new(Char::Id).integer_len(11).not_null().auto_increment().primary_key())
            .col(ColumnDef::new(Char::FontSize).integer_len(11).not_null())
            .col(ColumnDef::new(Char::Character).string_len(255).not_null())
            .col(ColumnDef::new(Char::SizeW).integer_len(11).not_null())
            .col(ColumnDef::new(Char::SizeH).integer_len(11).not_null())
            .col(ColumnDef::new(Char::FontId).integer_len(11).default(Value::Null))
            .foreign_key(
                ForeignKey::create()
                    .name("FK_2e303c3a712662f1fc2a4d0aad6")
                    .from(Char::Table, Char::FontId)
                    .to(Font::Table, Font::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Restrict)
            )
            .engine("InnoDB")
            .character_set("utf8mb4")
            .collate("utf8mb4_unicode_ci")
            .to_string(MysqlQueryBuilder),
        vec![
            "CREATE TABLE IF NOT EXISTS `character` (",
                "`id` int(11) NOT NULL AUTO_INCREMENT PRIMARY KEY,",
                "`font_size` int(11) NOT NULL,",
                "`character` varchar(255) NOT NULL,",
                "`size_w` int(11) NOT NULL,",
                "`size_h` int(11) NOT NULL,",
                "`font_id` int(11) DEFAULT NULL,",
                "CONSTRAINT `FK_2e303c3a712662f1fc2a4d0aad6`",
                    "FOREIGN KEY (`font_id`) REFERENCES `font` (`id`)",
                    "ON DELETE CASCADE ON UPDATE RESTRICT",
            ") ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci",
        ].join(" ")
    );
}

#[test]
fn create_4() {
    assert_eq!(
        Table::create()
            .table(Glyph::Table)
            .col(ColumnDef::new(Glyph::Id).integer().not_null().extra("ANYTHING I WANT TO SAY".to_owned()))
            .to_string(MysqlQueryBuilder),
        vec![
            "CREATE TABLE `glyph` (",
                "`id` int NOT NULL ANYTHING I WANT TO SAY",
            ")",
        ].join(" ")
    );
}

#[test]
fn create_5() {
    assert_eq!(
        Table::create()
            .table(Glyph::Table)
            .col(ColumnDef::new(Glyph::Id).integer().not_null())
            .index(
                Index::create()
                    .unique()
                    .name("idx-glyph-id")
                    .col(Glyph::Id)
            )
            .to_string(MysqlQueryBuilder),
        vec![
            "CREATE TABLE `glyph` (",
                "`id` int NOT NULL,",
                "UNIQUE KEY `idx-glyph-id` (`id`)",
            ")",
        ].join(" ")
    );
}

#[test]
fn drop_1() {
    assert_eq!(
        Table::drop()
            .table(Glyph::Table)
            .table(Char::Table)
            .cascade()
            .to_string(MysqlQueryBuilder),
        "DROP TABLE `glyph`, `character` CASCADE"
    );
}

#[test]
fn truncate_1() {
    assert_eq!(
        Table::truncate()
            .table(Font::Table)
            .to_string(MysqlQueryBuilder),
        "TRUNCATE TABLE `font`"
    );
}

#[test]
fn alter_1() {
    assert_eq!(
        Table::alter()
            .table(Font::Table)
            .add_column(ColumnDef::new(Alias::new("new_col")).integer().not_null().default(100))
            .to_string(MysqlQueryBuilder),
        "ALTER TABLE `font` ADD COLUMN `new_col` int NOT NULL DEFAULT 100"
    );
}

#[test]
fn alter_2() {
    assert_eq!(
        Table::alter()
            .table(Font::Table)
            .modify_column(ColumnDef::new(Alias::new("new_col")).big_integer().default(999))
            .to_string(MysqlQueryBuilder),
        "ALTER TABLE `font` MODIFY COLUMN `new_col` bigint DEFAULT 999"
    );
}

#[test]
fn alter_3() {
    assert_eq!(
        Table::alter()
            .table(Font::Table)
            .rename_column(Alias::new("new_col"), Alias::new("new_column"))
            .to_string(MysqlQueryBuilder),
        "ALTER TABLE `font` RENAME COLUMN `new_col` TO `new_column`"
    );
}

#[test]
fn alter_4() {
    assert_eq!(
        Table::alter()
            .table(Font::Table)
            .drop_column(Alias::new("new_column"))
            .to_string(MysqlQueryBuilder),
        "ALTER TABLE `font` DROP COLUMN `new_column`"
    );
}

#[test]
fn alter_5() {
    assert_eq!(
        Table::rename()
            .table(Font::Table, Alias::new("font_new"))
            .to_string(MysqlQueryBuilder),
        "RENAME TABLE `font` TO `font_new`"
    );
}

#[test]
#[should_panic(expected = "No alter option found")]
fn alter_6() {
    Table::alter().to_string(MysqlQueryBuilder);
}