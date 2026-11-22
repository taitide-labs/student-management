mod error;
mod gateway;
mod types;

pub use error::{GatewayError, ProblemDetails};
pub use gateway::{Gateway, GatewayClient, StudentDetailPayload};
pub use types::{
    AccountSession, AccountStatus, AccountSummary, AmendGradeRequest, AssignTeachingRequest,
    BindPersonProfileRequest, ChangeAccountRoleRequest, CourseSummary, CreateCourseRequest,
    CreateHomeroomRequest, DisableAccountRequest, EnrollStudentRequest, EnrollmentChangeRecord,
    EnrollmentStatus, GradeAmendmentRecord, GradeEntrySummary, GradeStatus, HomeroomSummary,
    ProvisionAccountRequest, PublishGradesRequest, ReverseEnrollmentRequest, Role,
    SaveDraftGradeRequest, SignInRequest, StudentSummary, TeachingSummary, TransferHomeroomRequest,
    TransitionStudentRequest, UpdateHomeroomCapacityRequest,
};
