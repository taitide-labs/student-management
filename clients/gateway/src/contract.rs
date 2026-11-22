mod error;
mod gateway;

pub use error::GatewayError;
pub use gateway::{Gateway, GatewayClient, GatewayFuture};
pub use student_management_contracts::{
    AccountSession, AccountStatus, AccountSummary, AmendGradeRequest, AssignTeachingRequest,
    BindPersonProfileRequest, ChangeAccountRoleRequest, CourseSummary, CreateCourseRequest,
    CreateHomeroomRequest, DisableAccountRequest, EnrollStudentRequest, EnrollmentChangeRecord,
    EnrollmentStatus, GradeAmendmentRecord, GradeEntrySummary, GradeStatus, HomeroomSummary,
    ProblemDetails, ProvisionAccountRequest, PublishGradesRequest, ReverseEnrollmentRequest, Role,
    SaveDraftGradeRequest, SignInRequest, StudentDetailPayload, StudentSummary, TeachingSummary,
    TransferHomeroomRequest, TransitionStudentRequest, UpdateHomeroomCapacityRequest,
};
