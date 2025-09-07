/// A trait for all price waiting operations to enable a priority list for maximum execution
pub trait PriceWaitingOperation {
    /// excutes the paritucular operation on the market
    fn execute(&self);

    //  fn executor(&self) -> Principal;
}
