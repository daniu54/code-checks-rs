use owo_colors::OwoColorize as _;

use annotate_snippets::{Level, Renderer, Snippet};
use csharp_tree_sitter_utils_rs::ExtendedTree;

use crate::{
    check_error::CheckError,
    contexts::{CheckContext, FileContext},
    filters::CheckFilter,
};

use super::Check;

pub struct CheckAsyncForTask {}

impl<'c> Check<'c> for CheckAsyncForTask {
    fn execute_check(
        &'c self,
        context: &'c CheckContext,
        filters: &'c [&'c CheckFilter],
    ) -> Vec<CheckError> {
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

        for filter in filters {
            match filter {
                CheckFilter::IgnoreByFilePathRegex(regex) => {
                    if regex.is_match(file_path) {
                        return Vec::new();
                    }
                }
                CheckFilter::IgnoreByNameSpaceRegex(regex) => {
                    let namespaces_names = tree
                        .into_iter()
                        .filter(|n| n.ts_node.kind() == "namespace_declaration")
                        .map(|n| n.ts_node.child_by_field_name("name").unwrap())
                        .map(|n| file_contents[n.byte_range()].to_string());

                    // NOTE tree sitter does not parse file scoped namespaces
                    // e.g. `namespace test;`
                    let probable_file_namespaces = tree
                        .into_iter()
                        .filter(|n| n.ts_node.kind() == "ERROR")
                        .filter(|n| n.source_code.trim().starts_with("namespace"))
                        .map(|n| {
                            n.source_code
                                .trim_start_matches("namespace ")
                                .trim_end_matches(';')
                                .to_string()
                        });

                    if namespaces_names
                        .chain(probable_file_namespaces)
                        .any(|n| regex.is_match(&n))
                    {
                        return Vec::new();
                    }
                }
            }
        }

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
        filters::CheckFilter,
    };

    #[test]
    fn check_should_succeed() {
        let contents = vec![
            r#"public Task CorrectAsync() { }"#,
            r#" public Task CorrectAsync(Some arguments) { }"#,
            r#"public async Task Correct2Async() { }"#,
        ];

        let filters = vec![];

        for file_contents in contents {
            let context = CheckContext::File(FileContext {
                file_path: "some/file.cs",
                file_contents,
            });

            let check = CheckAsyncForTask {};

            let errors = check.execute_check(&context, &filters);

            assert!(errors.is_empty());
        }
    }

    #[test]
    fn check_should_return_errors() {
        let contents = vec![
            r#"public Task Incorrect1() { } "#,
            r#"  public Task Incorrect1() { } "#,
            r#"public Task Incorrect1(Async) { } "#,
            r#"public async Task Incorrect2() { } "#,
            r#"public async Task IncorrectAsync3() { } "#,
            r#"public Task IncorrectAsync4() { }"#,
        ];

        let filters = vec![];

        for file_contents in contents {
            let context = CheckContext::File(FileContext {
                file_path: "some/file.cs",
                file_contents,
            });

            let check = CheckAsyncForTask {};

            let errors = check.execute_check(&context, &filters);

            assert!(
                errors.len() == 1,
                "File should contain errors\n---\n{}\n---\n",
                file_contents
            );

            let regex = Regex::new(r#"consider.*Async"#).unwrap();

            assert_eq!(true, regex.is_match(&errors[0].message));
        }
    }

    #[test]
    fn check_should_ignore_namespaces() {
        let contents = vec![
            r#"
                namespace ignore_me {
                    public Task IncorrectButIgnored() { }
                }
            "#,
            r#"
                namespace ignore_me;
                public Task IncorrectButIgnored() { }
            "#,
            r#"
                namespace prefix_ignore_me;
                public Task IncorrectButIgnored() { }
            "#,
            r#"
                namespace ignore_me_postfix;
                public Task IncorrectButIgnored() { }
            "#,
        ];

        let namespace_regex = Regex::new(r#"ignore_me"#).unwrap();

        let ignore_namespace = CheckFilter::IgnoreByNameSpaceRegex(&namespace_regex);

        let filters = vec![&ignore_namespace];

        for file_contents in contents {
            let context = CheckContext::File(FileContext {
                file_path: "some/file.cs",
                file_contents,
            });

            let check = CheckAsyncForTask {};

            let errors = check.execute_check(&context, &filters);

            assert!(errors.is_empty());
        }
    }

    #[test]
    fn check_should_ignore_file_paths() {
        let contents = vec![
            r#"
                public Task IncorrectButIgnored() { }
            "#,
        ];

        let file_name = "some/FileTests.cs";

        let filepath_regex = Regex::new(r#".*Tests\.cs"#).unwrap();

        assert!(filepath_regex.is_match(file_name));

        let ignore_filepath = CheckFilter::IgnoreByFilePathRegex(&filepath_regex);

        let filters = vec![&ignore_filepath];

        for file_contents in contents {
            let context = CheckContext::File(FileContext {
                file_path: file_name,
                file_contents,
            });

            let check = CheckAsyncForTask {};

            let errors = check.execute_check(&context, &filters);

            assert!(errors.is_empty());
        }
    }
}
