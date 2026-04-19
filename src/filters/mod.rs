use ethers::types::Transaction;

pub type Pred = Box<dyn Fn(&Transaction) -> bool + Send + Sync>;

#[derive(Default)]
pub struct Filters(Vec<Pred>);

impl Filters {
    pub fn push(&mut self, f: impl Fn(&Transaction) -> bool + Send + Sync + 'static) {
        self.0.push(Box::new(f));
    }

    pub fn apply(&self, tx: &Transaction) -> bool {
        self.0.iter().all(|f| f(tx))
    }
}

pub fn range<T, F>(filters: &mut Filters, gt: Option<T>, lt: Option<T>, extract: F)
where
    T: PartialOrd + Copy + Send + Sync + 'static,
    F: Fn(&Transaction) -> Option<T> + Send + Sync + 'static,
{
    if gt.is_none() && lt.is_none() {
        return;
    }
    filters.push(move |tx| {
        let Some(v) = extract(tx) else { return false };
        gt.is_none_or(|g| v >= g) && lt.is_none_or(|l| v <= l)
    });
}
