use super::{DirectoryContext, FileContext, GitContext};

pub enum CheckContext<'c> {
    File(FileContext<'c>),
    Directory(DirectoryContext<'c>),
    Git(GitContext<'c>),
}
