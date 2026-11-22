use student_management_api::build_router;
use student_management_contracts::{AccountSession, OperatorPrincipal, SignInRequest};
use student_management_runtime::{LocalBackend, build_app_state};
use tokio::{net::TcpListener, task::JoinHandle};

pub struct LiveApiServer {
    pub base_url: String,
    pub backend: LocalBackend,
    pub admin_session: AccountSession,
    handle: JoinHandle<()>,
}

impl Drop for LiveApiServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

impl LiveApiServer {
    #[must_use]
    pub fn admin_principal(&self) -> OperatorPrincipal {
        OperatorPrincipal {
            account_id: self.admin_session.account_id.clone(),
            display_name: self.admin_session.display_name.clone(),
            role: self.admin_session.role,
            bound_student_no: self.admin_session.bound_student_no.clone(),
        }
    }
}

pub async fn start_live_server() -> LiveApiServer {
    let state = build_app_state("sqlite::memory:")
        .await
        .expect("build runtime state");
    let backend = LocalBackend::new(state.clone());
    backend
        .ensure_bootstrap_admin("admin", "系统管理员", "demo")
        .await
        .expect("bootstrap admin");
    let admin_session = backend
        .sign_in(SignInRequest {
            username: "admin".to_owned(),
            password: "demo".to_owned(),
        })
        .await
        .expect("sign in bootstrap admin");

    let router = build_router(state);
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ephemeral port");
    let addr = listener.local_addr().expect("local addr");
    let handle = tokio::spawn(async move {
        let _res = axum::serve(listener, router).await;
    });

    LiveApiServer {
        base_url: format!("http://{addr}"),
        backend,
        admin_session,
        handle,
    }
}
