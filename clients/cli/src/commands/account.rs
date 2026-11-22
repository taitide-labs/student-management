use clap::{Arg, ArgMatches, Command};
use student_management_gateway::contract::{
    AccountSession, BindPersonProfileRequest, ChangeAccountRoleRequest, DisableAccountRequest,
    Gateway, ProvisionAccountRequest,
};

use super::shared::{
    emit_error, emit_json_or_text, parse_role, parse_u64, require_yes, str_arg, usage_err,
};

#[must_use]
pub fn command() -> Command {
    Command::new("account")
        .about("管理登录账号、角色与档案绑定")
        .subcommand_required(true)
        .subcommand(Command::new("list").about("查询账号列表"))
        .subcommand(
            Command::new("provision")
                .about("开通新登录账号")
                .arg(Arg::new("username").long("username").required(true))
                .arg(Arg::new("display_name").long("display-name").required(true))
                .arg(Arg::new("role").long("role").required(true))
                .arg(Arg::new("student_no").long("student-no")),
        )
        .subcommand(
            Command::new("bind")
                .about("绑定或解绑学生学号")
                .arg(Arg::new("account_id").required(true))
                .arg(Arg::new("student_no").long("student-no"))
                .arg(Arg::new("version").long("version").required(true)),
        )
        .subcommand(
            Command::new("role")
                .about("变更账号角色")
                .arg(Arg::new("account_id").required(true))
                .arg(Arg::new("role").long("role").required(true))
                .arg(Arg::new("version").long("version").required(true)),
        )
        .subcommand(
            Command::new("disable")
                .about("停用账号")
                .arg(Arg::new("account_id").required(true))
                .arg(Arg::new("reason").long("reason").required(true))
                .arg(Arg::new("version").long("version").required(true)),
        )
}

pub async fn execute<G: Gateway>(
    gw: &G,
    session: &AccountSession,
    sub: &ArgMatches,
    json: bool,
    yes: bool,
) -> u8 {
    match sub.subcommand() {
        Some(("list", _)) => match gw.list_accounts(session).await {
            Ok(rows) => emit_json_or_text(&rows, json, &format!("共 {} 个账号", rows.len())),
            Err(err) => emit_error(&err),
        },
        Some(("provision", args)) => {
            let username = str_arg(args, "username");
            let display_name = str_arg(args, "display_name");
            let Some(role) = parse_role(str_arg(args, "role")) else {
                return usage_err("无效角色，请传入 admin / teacher / student。");
            };
            let req = ProvisionAccountRequest {
                username: username.to_owned(),
                display_name: display_name.to_owned(),
                role,
                bound_student_no: args.get_one::<String>("student_no").cloned(),
                idempotency_key: format!("cli-prov-{username}"),
            };
            match gw.provision_account(session, &req).await {
                Ok(created) => emit_json_or_text(&created, json, &created.username),
                Err(err) => emit_error(&err),
            }
        }
        Some(("bind", args)) => {
            let account_id = str_arg(args, "account_id");
            let version = parse_u64(args, "version");
            let req = BindPersonProfileRequest {
                student_no: args.get_one::<String>("student_no").cloned(),
                version,
                idempotency_key: format!("cli-bind-{account_id}-{version}"),
            };
            match gw.bind_person_profile(session, account_id, &req).await {
                Ok(updated) => emit_json_or_text(&updated, json, &updated.account_id),
                Err(err) => emit_error(&err),
            }
        }
        Some(("role", args)) => {
            let account_id = str_arg(args, "account_id");
            let Some(role) = parse_role(str_arg(args, "role")) else {
                return usage_err("无效角色，请传入 admin / teacher / student。");
            };
            let version = parse_u64(args, "version");
            let req = ChangeAccountRoleRequest {
                role,
                version,
                idempotency_key: format!("cli-role-{account_id}-{version}"),
            };
            match gw.change_account_role(session, account_id, &req).await {
                Ok(updated) => emit_json_or_text(&updated, json, updated.role.label()),
                Err(err) => emit_error(&err),
            }
        }
        Some(("disable", args)) => {
            if let Some(code) = require_yes(yes, "disable") {
                return code;
            }
            let account_id = str_arg(args, "account_id");
            let reason = str_arg(args, "reason");
            let version = parse_u64(args, "version");
            let req = DisableAccountRequest {
                reason: reason.to_owned(),
                version,
                idempotency_key: format!("cli-dis-{account_id}-{version}"),
            };
            match gw.disable_account(session, account_id, &req).await {
                Ok(updated) => emit_json_or_text(&updated, json, updated.status.label()),
                Err(err) => emit_error(&err),
            }
        }
        _ => 2,
    }
}
