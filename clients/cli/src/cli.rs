use clap::{Arg, ArgAction, Command};

use crate::commands;

#[must_use]
pub fn build_command() -> Command {
    Command::new("sms")
        .version(env!("CARGO_PKG_VERSION"))
        .about("学生管理系统命令行工具")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .arg(
            Arg::new("server")
                .long("server")
                .env("SMS_SERVER")
                .global(true)
                .value_name("URL")
                .help("服务端基址"),
        )
        .arg(
            Arg::new("local")
                .long("local")
                .global(true)
                .action(ArgAction::SetTrue)
                .help("启用本地嵌入式运行时"),
        )
        .arg(
            Arg::new("token")
                .long("token")
                .env("SMS_TOKEN")
                .global(true)
                .value_name("TOKEN")
                .help("认证访问令牌"),
        )
        .arg(
            Arg::new("json")
                .long("json")
                .global(true)
                .action(ArgAction::SetTrue)
                .help("以 JSON 格式输出至标准输出"),
        )
        .arg(
            Arg::new("yes")
                .long("yes")
                .global(true)
                .action(ArgAction::SetTrue)
                .help("跳过破坏性操作交互确认"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .global(true)
                .action(ArgAction::SetTrue)
                .help("静默非错误提示输出"),
        )
        .subcommand(commands::auth::command())
        .subcommand(commands::account::command())
        .subcommand(commands::homeroom::command())
        .subcommand(commands::student::command())
        .subcommand(commands::grading::course_command())
        .subcommand(commands::grading::teaching_command())
        .subcommand(commands::grading::grade_command())
}
