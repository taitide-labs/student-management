#[cfg(test)]
#[path = "acceptance/support.rs"]
mod support;

#[cfg(test)]
#[path = "acceptance/cmd_sign_in.rs"]
mod cmd_sign_in;

#[cfg(test)]
#[path = "acceptance/cmd_provision_account.rs"]
mod cmd_provision_account;

#[cfg(test)]
#[path = "acceptance/cmd_bind_person_profile.rs"]
mod cmd_bind_person_profile;

#[cfg(test)]
#[path = "acceptance/cmd_change_account_role.rs"]
mod cmd_change_account_role;

#[cfg(test)]
#[path = "acceptance/cmd_disable_account.rs"]
mod cmd_disable_account;
