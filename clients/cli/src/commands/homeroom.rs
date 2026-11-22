use clap::{Arg, ArgMatches, Command};
use student_management_gateway::contract::{
    AccountSession, CreateHomeroomRequest, Gateway, UpdateHomeroomCapacityRequest,
};

use super::shared::{emit_error, emit_json_or_text, make_idempotency_key, parse_u64, str_arg};

#[must_use]
pub fn command() -> Command {
    Command::new("homeroom")
        .about("管理行政班与额定容量")
        .subcommand_required(true)
        .subcommand(Command::new("list").about("查询行政班列表"))
        .subcommand(
            Command::new("create")
                .about("创建行政班")
                .arg(Arg::new("name").long("name").required(true))
                .arg(Arg::new("grade_year").long("grade-year").required(true))
                .arg(Arg::new("capacity").long("capacity")),
        )
        .subcommand(
            Command::new("capacity")
                .about("调整行政班额定容量")
                .arg(Arg::new("homeroom_id").required(true))
                .arg(Arg::new("capacity").long("capacity"))
                .arg(Arg::new("version").long("version").required(true)),
        )
}

pub async fn execute<G: Gateway>(
    gw: &G,
    session: &AccountSession,
    sub: &ArgMatches,
    json: bool,
) -> u8 {
    match sub.subcommand() {
        Some(("list", _)) => match gw.list_homerooms().await {
            Ok(rooms) => emit_json_or_text(&rooms, json, &format!("共 {} 个行政班", rooms.len())),
            Err(err) => emit_error(&err),
        },
        Some(("create", args)) => {
            let name = str_arg(args, "name");
            let grade_year = str_arg(args, "grade_year").parse::<u16>().unwrap_or(0);
            let capacity = args
                .get_one::<String>("capacity")
                .and_then(|c| c.parse::<u32>().ok());
            let req = CreateHomeroomRequest {
                name: name.to_owned(),
                grade_year,
                capacity,
                idempotency_key: make_idempotency_key("cli-hr-create", name),
            };
            match gw.create_homeroom(session, &req).await {
                Ok(created) => emit_json_or_text(&created, json, &created.name),
                Err(err) => emit_error(&err),
            }
        }
        Some(("capacity", args)) => {
            let homeroom_id = str_arg(args, "homeroom_id");
            let capacity = args
                .get_one::<String>("capacity")
                .and_then(|c| c.parse::<u32>().ok());
            let version = parse_u64(args, "version");
            let req = UpdateHomeroomCapacityRequest {
                capacity,
                version,
                idempotency_key: format!("cli-hr-cap-{homeroom_id}-{version}"),
            };
            match gw
                .update_homeroom_capacity(session, homeroom_id, &req)
                .await
            {
                Ok(updated) => emit_json_or_text(&updated, json, &updated.name),
                Err(err) => emit_error(&err),
            }
        }
        _ => 2,
    }
}
