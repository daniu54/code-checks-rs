use crate::check_error::CheckError;
use crate::contexts::CheckContext;
use crate::filters::CheckFilter;

pub trait Check<'c> {
    fn execute_check(
        &'c self,
        context: &'c CheckContext,
        filters: &'c [&'c CheckFilter],
    ) -> Vec<CheckError>;
}
