use clap::{Arg, Command};

#[must_use]
pub fn command() -> Command {
    Command::new("auth")
        .about("管理登录会话与认证令牌")
        .subcommand_required(true)
        .subcommand(
            Command::new("login")
                .about("登录获取访问令牌")
                .arg(Arg::new("username").long("username").required(true))
                .arg(Arg::new("password").long("password").required(true)),
        )
        .subcommand(Command::new("whoami").about("查看当前登录会话"))
        .subcommand(Command::new("logout").about("清除本地会话令牌"))
}
