use proptest::prelude::*;
use student_management_contracts::{AccountStatus, Role};
use student_management_primitives::{
    NonEmptyText, PositiveVersion, ReasonText, StudentNo, Username,
};

use crate::{Account, AccountProvisionInput, IdentityDomainError};

fn v1() -> PositiveVersion {
    PositiveVersion::try_new(1).expect("version 1")
}

#[test]
fn non_student_account_cannot_bind_student_number() {
    let teacher = Account::provision(AccountProvisionInput {
        account_id: "acc-t1".to_owned(),
        username: Username::try_new("lin_laoshi").expect("username"),
        display_name: NonEmptyText::try_new("姓名", "林老师", 32).expect("name"),
        role: Role::Teacher,
        bound_student_no: Some(StudentNo::try_new("20261001").expect("student no")),
    });
    assert_eq!(
        teacher,
        Err(IdentityDomainError::StudentBindingRequiresStudentRole)
    );
}

#[test]
fn role_change_from_student_clears_student_binding_and_blocks_self_disable() {
    let mut account = Account::provision(AccountProvisionInput {
        account_id: "acc-s1".to_owned(),
        username: Username::try_new("stu_01").expect("username"),
        display_name: NonEmptyText::try_new("姓名", "张同学", 32).expect("name"),
        role: Role::Student,
        bound_student_no: Some(StudentNo::try_new("20261001").expect("student no")),
    })
    .expect("provision student");

    account
        .change_role(v1(), Role::Teacher)
        .expect("promote to teacher");
    assert_eq!(account.bound_student_no, None);
    assert_eq!(account.version.get(), 2);

    let self_disable = account.disable(
        "acc-s1",
        account.version,
        ReasonText::try_new("测试自停用").expect("reason"),
    );
    assert_eq!(self_disable, Err(IdentityDomainError::SelfDisableForbidden));

    account
        .disable(
            "acc-admin",
            account.version,
            ReasonText::try_new("离职停用").expect("reason"),
        )
        .expect("admin disables account");
    assert_eq!(account.status, AccountStatus::Disabled);
}

proptest! {
    #[test]
    fn disabled_account_never_mutates_role_or_binding(role_idx in 0u8..3u8) {
        let mut account = Account::provision(AccountProvisionInput {
            account_id: "acc-1".to_owned(),
            username: Username::try_new("user_01").expect("username"),
            display_name: NonEmptyText::try_new("姓名", "用户", 32).expect("name"),
            role: Role::Student,
            bound_student_no: None,
        }).expect("provision");
        account
            .disable("acc-admin", account.version, ReasonText::try_new("毕业离校").expect("reason"))
            .expect("disable");

        let target_role = match role_idx {
            0 => Role::Admin,
            1 => Role::Teacher,
            _ => Role::Student,
        };
        let err = account.change_role(account.version, target_role);
        prop_assert_eq!(err, Err(IdentityDomainError::DisabledAccountImmutable));
    }
}
