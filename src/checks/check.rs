use crate::check_error::CheckError;
use crate::contexts::CheckContext;

pub trait Check<'c> {
    fn execute_check(&'c self, context: &'c CheckContext) -> Vec<CheckError>;
}
