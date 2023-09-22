use crate::model::data_record::DataRecord;
use crate::physical::exec::Exec;
use datafusion_expr::Expr;

pub struct ExecProjection {
    // Physical plan input into this Exec for example
    // SourceScan, CsvScan, SqliteTableScan, etc.
    pub(crate) input: Box<dyn Exec>,
    // expressions to be projected on the returned row
    pub(crate) expressions: Vec<Expr>,
}

impl Exec for ExecProjection {
    fn execute(&self) -> Vec<DataRecord> {
        // why use into_iter() here instead of iter()?
        // https://www.becomebetterprogrammer.com/rust-iter-vs-iter_mut-vs-into_iter/
        // into_iter() yields any of T (moved value), &T or &mut T depending on the usage
        // seems like we need a moved value here, not sure why we need yet.
        self.input
            .execute()
            .into_iter()
            .map(|record| return record)
            .collect()
    }
}