mod assist;
#[cfg(feature = "dev-demo")]
mod mock;

pub use assist::EditorAssistService;
#[cfg(feature = "dev-demo")]
pub use mock::MockTextConvertor;
