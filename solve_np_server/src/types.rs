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

#[derive(Hash, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Deserialize)]
pub enum ProblemId {
    #[serde(rename = "travelling_salesman")]
    TravellingSalesman,
    #[serde(rename = "graph_coloring")]
    GraphColoring,
    #[serde(rename = "vertex_cover")]
    VertexCover,
    #[serde(rename = "maximum_clique")]
    MaximumClique,
    #[serde(rename = "maximum_cut")]
    MaximumCut,
    #[serde(rename = "steiner_tree")]
    SteinerTree,
}

impl fmt::Display for ProblemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProblemId::TravellingSalesman => write!(f, "travelling_salesman"),
            ProblemId::GraphColoring => write!(f, "graph_coloring"),
            ProblemId::VertexCover => write!(f, "vertex_cover"),
            ProblemId::MaximumClique => write!(f, "maximum_clique"),
            ProblemId::MaximumCut => write!(f, "maximum_cut"),
            ProblemId::SteinerTree => write!(f, "steiner_tree"),
        }
    }
}

impl FromSql for ProblemId {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value.as_str()? {
            "travelling_salesman" => Ok(Self::TravellingSalesman),
            "graph_coloring" => Ok(Self::GraphColoring),
            "vertex_cover" => Ok(Self::VertexCover),
            "maximum_clique" => Ok(Self::MaximumClique),
            "maximum_cut" => Ok(Self::MaximumCut),
            "steiner_tree" => Ok(Self::SteinerTree),
            _ => Err(rusqlite::types::FromSqlError::OutOfRange(4)),
        }
    }
}

impl ToSql for ProblemId {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        match self {
            Self::TravellingSalesman => ("travelling_salesman").to_sql(),
            Self::GraphColoring => ("graph_coloring").to_sql(),
            Self::VertexCover => ("vertex_cover").to_sql(),
            Self::MaximumClique => ("maximum_clique").to_sql(),
            Self::MaximumCut => ("maximum_cut").to_sql(),
            Self::SteinerTree => ("steiner_tree").to_sql(),
        }
    }
}

#[derive(Deserialize, Ord, PartialOrd, Hash, Eq, PartialEq, Clone)]
pub struct InputId(String);

impl<T: Into<String>> From<T> for InputId {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

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

impl FromSql for InputId {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        Ok(value.as_str()?.into())
    }
}

#[derive(Clone, Deserialize, Hash, PartialEq, Eq)]
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
