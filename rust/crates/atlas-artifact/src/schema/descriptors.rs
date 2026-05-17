#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Table {
    name: &'static str,
}

impl Table {
    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }

    pub const fn name(self) -> &'static str {
        self.name
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Column {
    table: Table,
    name: &'static str,
}

impl Column {
    pub const fn new(table: Table, name: &'static str) -> Self {
        Self { table, name }
    }

    pub const fn table(self) -> Table {
        self.table
    }

    pub const fn name(self) -> &'static str {
        self.name
    }

    pub fn key(self) -> String {
        format!("{}.{}", self.table.name(), self.name)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TableKind {
    Ordinary,
    Fts5 { unindexed: &'static [Column] },
    Vector,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SqlType {
    Text,
    Integer,
    Real,
    Blob,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColumnCheck {
    Boolean,
    ClosedSet(&'static [&'static str]),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForeignKeyAction {
    Cascade,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TableConstraint {
    PrimaryKey(&'static [Column]),
    ForeignKey {
        columns: &'static [Column],
        target_table: Table,
        target_columns: &'static [Column],
        on_delete: Option<ForeignKeyAction>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ColumnDescriptor {
    pub column: Column,
    pub sql_type: SqlType,
    pub nullable: bool,
    pub primary_key: bool,
    pub check: Option<ColumnCheck>,
}

impl ColumnDescriptor {
    pub const fn new(column: Column, sql_type: SqlType) -> Self {
        Self {
            column,
            sql_type,
            nullable: true,
            primary_key: false,
            check: None,
        }
    }

    pub const fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }

    pub const fn primary_key(mut self) -> Self {
        self.primary_key = true;
        self.nullable = false;
        self
    }

    pub const fn check(mut self, check: ColumnCheck) -> Self {
        self.check = Some(check);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TableDescriptor {
    pub table: Table,
    pub kind: TableKind,
    pub column_descriptors: &'static [ColumnDescriptor],
    pub table_constraints: &'static [TableConstraint],
    pub validate_columns: bool,
}

impl TableDescriptor {
    pub const fn ordinary(
        table: Table,
        column_descriptors: &'static [ColumnDescriptor],
        table_constraints: &'static [TableConstraint],
    ) -> Self {
        Self {
            table,
            kind: TableKind::Ordinary,
            column_descriptors,
            table_constraints,
            validate_columns: true,
        }
    }

    pub const fn fts5(
        table: Table,
        column_descriptors: &'static [ColumnDescriptor],
        unindexed: &'static [Column],
    ) -> Self {
        Self {
            table,
            kind: TableKind::Fts5 { unindexed },
            column_descriptors,
            table_constraints: &[],
            validate_columns: true,
        }
    }

    pub fn columns(self) -> Vec<Column> {
        self.column_descriptors
            .iter()
            .map(|descriptor| descriptor.column)
            .collect()
    }
}

pub const fn text(column: Column) -> ColumnDescriptor {
    ColumnDescriptor::new(column, SqlType::Text)
}

pub const fn text_not_null(column: Column) -> ColumnDescriptor {
    ColumnDescriptor::new(column, SqlType::Text).not_null()
}

pub const fn text_primary_key(column: Column) -> ColumnDescriptor {
    ColumnDescriptor::new(column, SqlType::Text).primary_key()
}

pub const fn integer(column: Column) -> ColumnDescriptor {
    ColumnDescriptor::new(column, SqlType::Integer)
}

pub const fn integer_not_null(column: Column) -> ColumnDescriptor {
    ColumnDescriptor::new(column, SqlType::Integer).not_null()
}

pub const fn real(column: Column) -> ColumnDescriptor {
    ColumnDescriptor::new(column, SqlType::Real)
}

pub const fn blob_not_null(column: Column) -> ColumnDescriptor {
    ColumnDescriptor::new(column, SqlType::Blob).not_null()
}

pub const fn boolean(column: Column) -> ColumnDescriptor {
    ColumnDescriptor::new(column, SqlType::Integer).check(ColumnCheck::Boolean)
}

pub const fn boolean_not_null(column: Column) -> ColumnDescriptor {
    ColumnDescriptor::new(column, SqlType::Integer)
        .not_null()
        .check(ColumnCheck::Boolean)
}

pub const fn closed_text_not_null(
    column: Column,
    values: &'static [&'static str],
) -> ColumnDescriptor {
    ColumnDescriptor::new(column, SqlType::Text)
        .not_null()
        .check(ColumnCheck::ClosedSet(values))
}

pub const fn primary_key(columns: &'static [Column]) -> TableConstraint {
    TableConstraint::PrimaryKey(columns)
}

pub const fn cascade_foreign_key(
    columns: &'static [Column],
    target_table: Table,
    target_columns: &'static [Column],
) -> TableConstraint {
    TableConstraint::ForeignKey {
        columns,
        target_table,
        target_columns,
        on_delete: Some(ForeignKeyAction::Cascade),
    }
}
