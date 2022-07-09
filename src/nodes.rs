use std::{
    any::Any,
    fmt::{self, Formatter},
    sync::Arc,
};

use datafusion::logical_plan::{Column, DFSchemaRef, Expr, LogicalPlan, UserDefinedLogicalNode};

#[derive(Debug)]
pub struct CreateTable {
    /// The table schema
    pub schema: DFSchemaRef,
    /// The table name
    pub name: String,
    /// Option to not error if table already exists
    pub if_not_exists: bool,
}

impl UserDefinedLogicalNode for CreateTable {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn inputs(&self) -> Vec<&LogicalPlan> {
        vec![]
    }

    fn schema(&self) -> &DFSchemaRef {
        // TODO or none?
        &self.schema
    }

    fn expressions(&self) -> Vec<Expr> {
        vec![]
    }

    fn fmt_for_explain(&self, _f: &mut Formatter) -> fmt::Result {
        todo!()
    }

    fn from_template(
        &self,
        _exprs: &[Expr],
        _inputs: &[LogicalPlan],
    ) -> std::sync::Arc<dyn UserDefinedLogicalNode> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Insert {
    /// The table name (TODO: should this be a table ref?)
    pub name: String,
    /// List of columns to set
    pub columns: Vec<Column>,
    /// Result of a query to insert
    pub input: Arc<LogicalPlan>,
}

impl UserDefinedLogicalNode for Insert {
    fn as_any(&self) -> &dyn Any {
        todo!()
    }

    fn inputs(&self) -> Vec<&LogicalPlan> {
        todo!()
    }

    fn schema(&self) -> &DFSchemaRef {
        todo!()
    }

    fn expressions(&self) -> Vec<Expr> {
        todo!()
    }

    fn fmt_for_explain(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        todo!()
    }

    fn from_template(
        &self,
        _exprs: &[Expr],
        _inputs: &[LogicalPlan],
    ) -> Arc<dyn UserDefinedLogicalNode> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Assignment {
    pub column: Column,
    pub expr: Expr,
}

#[derive(Debug)]
pub struct Update {
    /// The table name (TODO: should this be a table ref?)
    pub name: String,
    /// WHERE clause
    pub selection: Option<Expr>,
    /// Columns to update
    pub assignments: Vec<Assignment>,
}

impl UserDefinedLogicalNode for Update {
    fn as_any(&self) -> &dyn Any {
        todo!()
    }

    fn inputs(&self) -> Vec<&LogicalPlan> {
        todo!()
    }

    fn schema(&self) -> &DFSchemaRef {
        todo!()
    }

    fn expressions(&self) -> Vec<Expr> {
        todo!()
    }

    fn fmt_for_explain(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        todo!()
    }

    fn from_template(
        &self,
        _exprs: &[Expr],
        _inputs: &[LogicalPlan],
    ) -> Arc<dyn UserDefinedLogicalNode> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Delete {
    /// The table name (TODO: should this be a table ref?)
    pub name: String,
    /// WHERE clause
    pub selection: Option<Expr>,
}

impl UserDefinedLogicalNode for Delete {
    fn as_any(&self) -> &dyn Any {
        todo!()
    }

    fn inputs(&self) -> Vec<&LogicalPlan> {
        todo!()
    }

    fn schema(&self) -> &DFSchemaRef {
        todo!()
    }

    fn expressions(&self) -> Vec<Expr> {
        todo!()
    }

    fn fmt_for_explain(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        todo!()
    }

    fn from_template(
        &self,
        _exprs: &[Expr],
        _inputs: &[LogicalPlan],
    ) -> Arc<dyn UserDefinedLogicalNode> {
        todo!()
    }
}