use owo_colors::OwoColorize as _;

use annotate_snippets::{Level, Renderer, Snippet};
use csharp_tree_sitter_utils_rs::ExtendedTree;

use crate::{
    check_error::CheckError,
    contexts::{CheckContext, FileContext},
};

use super::Check;

pub struct CheckAsyncForTask {}

impl<'c> Check<'c> for CheckAsyncForTask {
    fn execute_check(&'c self, context: &'c CheckContext) -> Vec<CheckError> {
        let FileContext {
            file_path,
            file_contents,
        } = match context {
            CheckContext::File(file_context) => file_context,
            _ => return Vec::new(),
        };

        let mut errors = vec![];

        let renderer = Renderer::styled();

        let tree = ExtendedTree::from_source_code(file_contents);

        let method_nodes = tree.into_iter().filter(|n| {
            n.ts_node.kind() == "method_declaration"
                || n.ts_node.kind() == "local_function_statement"
        });

        let async_postfix = "Async";
        let task = "Task";
        let title = format!(
            "expected postfix {} for method that returns {}",
            async_postfix, task
        );
        let title_colored = format!(
            "expected postfix {} for method that returns {}",
            async_postfix.magenta(),
            task.magenta()
        );

        for node in method_nodes {
            let method_type_node = node.ts_node.child_by_field_name("type").unwrap();
            let method_name_node = node.ts_node.child_by_field_name("name").unwrap();

            let method_type = file_contents[method_type_node.byte_range()].to_string();
            let method_name = file_contents[method_name_node.byte_range()].to_string();

            let help = format!("consider {method_name}{}", async_postfix.cyan());

            if method_type.contains("Task") && !method_name.ends_with("Async") {
                let snippets = vec![Snippet::source(file_contents)
                    .origin(file_path)
                    .fold(true)
                    .annotation(
                        Level::Error
                            .span(method_name_node.byte_range())
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
                })
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use regex::Regex;

    use crate::{
        checks::{Check, CheckAsyncForTask},
        contexts::{CheckContext, FileContext},
    };

    #[test]
    fn test_successful_checking() {
        let contents = vec![
            r#"public Task CorrectAsync() { }"#,
            r#" public Task CorrectAsync(Some arguments) { }"#,
            r#"public async Task Correct2Async() { }"#,
        ];

        for file_contents in contents {
            let context = CheckContext::File(FileContext {
                file_path: "some/file.cs",
                file_contents,
            });

            let check = CheckAsyncForTask {};

            let errors = check.execute_check(&context);

            assert!(errors.is_empty());
        }
    }

    #[test]
    fn test_unsuccessful_checking() {
        let contents = vec![
            r#"public Task Incorrect1() { } "#,
            r#"  public Task Incorrect1() { } "#,
            r#"public Task Incorrect1(Async) { } "#,
            r#"public async Task Incorrect2() { } "#,
            r#"public async Task IncorrectAsync3() { } "#,
            r#"public Task IncorrectAsync4() { }"#,
        ];

        for file_contents in contents {
            let context = CheckContext::File(FileContext {
                file_path: "some/file.cs",
                file_contents,
            });

            let check = CheckAsyncForTask {};

            let errors = check.execute_check(&context);

            assert!(
                errors.len() == 1,
                "File should contain errors\n---\n{}\n---\n",
                file_contents
            );

            let regex = Regex::new(r#"consider.*Async"#).unwrap();

            assert_eq!(true, regex.is_match(&errors[0].message));
        }
    }
}
