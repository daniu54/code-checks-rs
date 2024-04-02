use owo_colors::OwoColorize as _;

use annotate_snippets::{Level, Renderer, Snippet};

use crate::{
    check_error::CheckError,
    contexts::{CheckContext, GitContext},
    filters::CheckFilter,
};

use super::Check;

pub struct CheckNoTodosInCommittedCode {}

impl<'c> Check<'c> for CheckNoTodosInCommittedCode {
    fn execute_check(
        &'c self,
        context: &'c CheckContext,
        filters: &'c [&'c CheckFilter],
    ) -> Vec<CheckError> {
        let GitContext { repository } = match context {
            CheckContext::Git(git_context) => git_context,
            _ => return Vec::new(),
        };

        let mut errors = vec![];

        let renderer = Renderer::styled();

        let commits = repository.get_commits_of_current_branch();

        for commit in commits {
            for file in commit.changed_files {
                for filter in filters {
                    if let CheckFilter::IgnoreByFilePathRegex(regex) = filter {
                        if regex.is_match(&file.file_path) {
                            return Vec::new();
                        }
                    }
                }

                let file_path = file.file_path;

                let changes_with_todos = file
                    .changes
                    .iter()
                    .filter(|c| c.content.contains("// TODO") || c.content.contains("// FIXME"));

                let todo_string = "TODO";
                let fixme_string = "FIXME";
                let commit_id = commit.commit.id();
                let commit_message = commit.commit.message().unwrap();

                let title = format!(
                    "expected no commited changes to contain {todo_string} or {fixme_string}"
                );

                let todo_string = "TODO".red();
                let fixme_string = "FIXME".red();

                let title_colored = format!(
                    "expected no commited changes to contain {todo_string} or {fixme_string}"
                );

                let help = format!("consider removing all {todo_string}s and {fixme_string}s form commit {} with message {:?}", commit_id.yellow(), commit_message.cyan());

                for change in changes_with_todos {
                    let file_contents = String::from_utf8(
                        repository
                            .repository
                            .find_blob(change.blob_oid)
                            .unwrap()
                            .content()
                            .to_vec(),
                    )
                    .unwrap();

                    let mut content_range_start = change.content_range.start;
                    let content_range_end = change.content_range.end - 1; // trim newline here

                    while content_range_start < content_range_end
                        && file_contents[content_range_start..content_range_end]
                            .chars()
                            .take_while(|c| c.is_whitespace())
                            .next()
                            .is_some()
                    {
                        content_range_start += 1;
                    }

                    let snippets = vec![Snippet::source(file_contents.as_str())
                        .origin(file_path.as_str())
                        .fold(true)
                        .annotation(
                            Level::Error
                                .span(content_range_start..content_range_end)
                                .label(&title),
                        )];

                    let footer = Level::Help.title(&help);

                    let message = Level::Error
                        .title(&title_colored)
                        .snippets(snippets)
                        .footer(footer);

                    errors.push(CheckError {
                        message: renderer.render(message).to_string(),
                        _context: context,
                    });
                }
            }
        }

        errors
    }
}
