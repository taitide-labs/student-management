use student_management_contracts::{ProblemDetails, SignInRequest};
use student_management_primitives::{NonEmptyText, Username};

#[derive(Debug, Clone)]
pub struct SignInCommand {
    pub username: Username,
    pub password: NonEmptyText,
}

impl TryFrom<SignInRequest> for SignInCommand {
    type Error = ProblemDetails;

    fn try_from(req: SignInRequest) -> Result<Self, Self::Error> {
        let username = Username::try_new(&req.username)
            .map_err(|err| ProblemDetails::bad_request("invalid_username", err.to_string()))?;
        let password = NonEmptyText::try_new("登录密码", &req.password, 128)
            .map_err(|err| ProblemDetails::bad_request("invalid_password", err.to_string()))?;
        Ok(Self { username, password })
    }
}
