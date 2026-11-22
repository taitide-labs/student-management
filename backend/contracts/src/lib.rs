pub mod clock;
pub mod error;
pub mod events;
pub mod ids;
pub mod ports;
pub mod protocol;
pub mod types;

pub use error::{PortError, ProblemDetails};
pub use events::IntegrationEvent;
pub use ports::{
    EnrollmentQueryPort, HomeroomSnapshot, IdentityLookupPort, OperatorPrincipal,
    StudentEnrollmentSnapshot, TeacherIdentity,
};
pub use protocol::{
    AccountSession, AccountSummary, AmendGradeRequest, AssignTeachingRequest,
    BindPersonProfileRequest, ChangeAccountRoleRequest, CourseSummary, CreateCourseRequest,
    CreateHomeroomRequest, DisableAccountRequest, EnrollStudentRequest, EnrollmentChangeRecord,
    GradeAmendmentRecord, GradeEntrySummary, HomeroomSummary, ProvisionAccountRequest,
    PublishGradesRequest, ReverseEnrollmentRequest, SaveDraftGradeRequest, SignInRequest,
    StudentDetailPayload, StudentSummary, TeachingSummary, TransferHomeroomRequest,
    TransitionStudentRequest, UpdateHomeroomCapacityRequest,
};
pub use types::{AccountStatus, EnrollmentStatus, GradeStatus, Role};
