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
