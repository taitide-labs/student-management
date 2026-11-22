use student_management_contracts::{
    AccountSession, AccountStatus, AccountSummary, AmendGradeRequest, AssignTeachingRequest,
    BindPersonProfileRequest, ChangeAccountRoleRequest, CourseSummary, CreateCourseRequest,
    CreateHomeroomRequest, DisableAccountRequest, EnrollStudentRequest, EnrollmentChangeRecord,
    EnrollmentStatus, GradeAmendmentRecord, GradeEntrySummary, GradeStatus, HomeroomSummary,
    ProblemDetails, ProvisionAccountRequest, PublishGradesRequest, ReverseEnrollmentRequest, Role,
    SaveDraftGradeRequest, SignInRequest, StudentDetailPayload, StudentSummary, TeachingSummary,
    TransferHomeroomRequest, TransitionStudentRequest, UpdateHomeroomCapacityRequest,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "学生管理系统 API",
        version = "0.1.0",
        description = "由后端切片派生并导出的南北向契约唯一事实源"
    ),
    paths(
        student_management_identity_features::cmd_sign_in::endpoint::sign_in,
        student_management_identity_features::qry_accounts::endpoint::list_accounts,
        student_management_identity_features::cmd_provision_account::endpoint::provision_account,
        student_management_identity_features::cmd_bind_person_profile::endpoint::bind_person_profile,
        student_management_identity_features::cmd_change_account_role::endpoint::change_account_role,
        student_management_identity_features::cmd_disable_account::endpoint::disable_account,
        student_management_enrollment_features::qry_homerooms::endpoint::list_homerooms,
        student_management_enrollment_features::cmd_create_homeroom::endpoint::create_homeroom,
        student_management_enrollment_features::cmd_update_homeroom_capacity::endpoint::update_homeroom_capacity,
        student_management_enrollment_features::qry_students::endpoint::list_students,
        student_management_enrollment_features::qry_student_detail::endpoint::student_detail,
        student_management_enrollment_features::cmd_enroll_student::endpoint::enroll_student,
        student_management_enrollment_features::cmd_transfer_homeroom::endpoint::transfer_homeroom,
        student_management_enrollment_features::cmd_suspend_student::endpoint::suspend_student,
        student_management_enrollment_features::cmd_resume_student::endpoint::resume_student,
        student_management_enrollment_features::cmd_withdraw_student::endpoint::withdraw_student,
        student_management_enrollment_features::cmd_graduate_student::endpoint::graduate_student,
        student_management_enrollment_features::cmd_reverse_enrollment::endpoint::reverse_enrollment,
        student_management_grading_features::qry_courses::endpoint::list_courses,
        student_management_grading_features::cmd_create_course::endpoint::create_course,
        student_management_grading_features::qry_teachings::endpoint::list_teachings,
        student_management_grading_features::cmd_assign_teaching::endpoint::assign_teaching,
        student_management_grading_features::qry_grades::endpoint::list_grades,
        student_management_grading_features::cmd_save_draft_grade::endpoint::save_draft_grade,
        student_management_grading_features::cmd_publish_grades::endpoint::publish_grades,
        student_management_grading_features::cmd_amend_grade::endpoint::amend_grade,
        student_management_grading_features::qry_grade_amendments::endpoint::list_grade_amendments,
    ),
    components(schemas(
        Role,
        AccountStatus,
        EnrollmentStatus,
        GradeStatus,
        ProblemDetails,
        AccountSession,
        AccountSummary,
        HomeroomSummary,
        StudentSummary,
        EnrollmentChangeRecord,
        StudentDetailPayload,
        CourseSummary,
        TeachingSummary,
        GradeEntrySummary,
        GradeAmendmentRecord,
        SignInRequest,
        EnrollStudentRequest,
        TransferHomeroomRequest,
        TransitionStudentRequest,
        ReverseEnrollmentRequest,
        CreateHomeroomRequest,
        UpdateHomeroomCapacityRequest,
        CreateCourseRequest,
        AssignTeachingRequest,
        SaveDraftGradeRequest,
        PublishGradesRequest,
        AmendGradeRequest,
        ProvisionAccountRequest,
        BindPersonProfileRequest,
        ChangeAccountRoleRequest,
        DisableAccountRequest,
    ))
)]
pub struct ApiDoc;

pub fn export_openapi_pretty() -> Result<String, serde_json::Error> {
    ApiDoc::openapi().to_pretty_json()
}
