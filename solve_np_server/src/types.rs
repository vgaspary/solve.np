use rusqlite::types::FromSql;
use rusqlite::types::ToSql;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;

pub type Score = usize;

#[derive(Clone, Copy, Serialize)]
pub enum Status {
    Untested,
    Attempted,
    Solved(Score),
}

impl ToSql for Status {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        match self {
            Self::Untested => (-2).to_sql(),
            Self::Attempted => (-1).to_sql(),
            Self::Solved(score) => score.to_sql(),
        }
    }
}

impl FromSql for Status {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value.as_i64()? {
            -2 => Ok(Self::Untested),
            -1 => Ok(Self::Attempted),
            n => Ok(Self::Solved(n as usize)),
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

impl ToSql for ProblemId {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

#[derive(Deserialize, Hash, Eq, PartialEq, Clone)]
pub struct InputId(String);

impl fmt::Display for InputId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl ToSql for InputId {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.0.to_sql()
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
