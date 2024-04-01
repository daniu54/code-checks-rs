use regex::Regex;

pub enum CheckFilter<'c> {
    IgnoreByFilePathRegex(&'c Regex),
    IgnoreByNameSpaceRegex(&'c Regex),
}
