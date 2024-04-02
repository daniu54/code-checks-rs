use git2_extensions::Repository;

pub struct GitContext<'c> {
    pub repository: Repository<'c>,
}
