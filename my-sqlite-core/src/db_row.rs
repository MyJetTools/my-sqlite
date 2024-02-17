use async_sqlite::rusqlite::{types::FromSql, Row};

pub struct DbRow<'s> {
    row: &'s Row<'s>,
    names: &'s [&'s str],
}

impl<'s> DbRow<'s> {
    pub fn new(row: &'s Row<'s>, names: &'s [&'s str]) -> Self {
        Self { row, names }
    }

    fn get_index(&self, name: &str) -> Option<usize> {
        let mut index = 0;
        for n in self.names {
            if *n == name {
                return Some(index);
            }

            index += 1;
        }

        None
    }

    pub fn get<T: FromSql>(&self, name: &str) -> T {
        let index = self.get_index(name);

        if index.is_none() {
            panic!("Column {} not found", name);
        }

        let index = index.unwrap();
        let result = self.row.get(index).unwrap();

        result
    }
}
