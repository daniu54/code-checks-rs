use crate::contexts::CheckContext;

pub struct CheckError<'c> {
    pub(crate) _context: &'c CheckContext<'c>,
    pub message: String,
}
