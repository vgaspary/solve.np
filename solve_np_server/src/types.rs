use rusqlite::types::FromSql;
use rusqlite::types::ToSql;
use serde::Deserialize;
use std::fmt;

pub type Score = usize;

#[derive(Clone, Copy)]
pub enum Status {
    Untested,
    Attempted,
    Solved(Score),
}

impl From<i64> for Status {
    fn from(value: i64) -> Self {
        match value {
            -1 => Status::Untested,
            -2 => Status::Attempted,
            score => Status::Solved(score as usize),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Ord, PartialOrd, Deserialize)]
pub struct ProblemId(String);

impl<T: Into<String>> From<T> for ProblemId {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl fmt::Display for ProblemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl FromSql for ProblemId {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        Ok(value.as_str()?.into())
    }
}

#[derive(Deserialize, Hash, Eq, PartialEq, Clone)]
pub struct InputId(String);

impl fmt::Display for InputId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Clone, Deserialize)]
pub struct SolutionId(String);

impl<T: Into<String>> From<T> for SolutionId {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl fmt::Display for SolutionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl FromSql for SolutionId {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        Ok(value.as_str()?.into())
    }
}

impl ToSql for SolutionId {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}
